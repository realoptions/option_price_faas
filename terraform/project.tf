# Filename: main.tf# Configure GCP project
provider "google" {
  project = var.project
}

terraform {
  backend "gcs" {
    bucket = "artifacts.finside.appspot.com"
    prefix = "terraform/state"
  }
}

resource "google_project_service" "container_registry" {
  service                    = "containerregistry.googleapis.com"
  disable_dependent_services = true
}

resource "google_project_service" "cloud_run" {
  service = "run.googleapis.com"
}

resource "google_project_service" "apigateway" {
  service = "apigateway.googleapis.com"
}

# actual app logic
resource "google_cloud_run_service" "realoptions" {
  name     = var.service_name
  location = var.region
  project  = var.project
  template {
    spec {
      containers {
        image = "gcr.io/${var.project}/${var.service_name}:${var.github_sha}"
        env {
          name  = "MAJOR_VERSION"
          value = var.version_major
        }
      }
    }
  }
  traffic {
    percent         = 100
    latest_revision = true
  }
  autogenerate_revision_name = true
  depends_on                 = [google_project_service.cloud_run]
}


# rapidapi logic 
resource "google_cloud_run_service" "realoptions_rapidapi" {
  name     = var.service_name_auth
  location = var.region
  project  = var.project
  template {
    spec {
      containers {
        image = "gcr.io/${var.project}/${var.service_name_auth}:${var.github_sha}"
        env {
          name  = "MAJOR_VERSION"
          value = var.version_major
        }
      }
    }
  }
  traffic {
    percent         = 100
    latest_revision = true
  }
  autogenerate_revision_name = true
  depends_on                 = [google_project_service.cloud_run]
}

# Enable public access on endpoints Cloud Run service
data "google_iam_policy" "noauth" {
  binding {
    role = "roles/run.invoker"
    members = [
      "allUsers",
    ]
  }
}
# Enable public access on endpoints Cloud Run service
resource "google_cloud_run_service_iam_policy" "realoptions_noauth" {
  location    = google_cloud_run_service.realoptions.location
  project     = google_cloud_run_service.realoptions.project
  service     = google_cloud_run_service.realoptions.name
  policy_data = data.google_iam_policy.noauth.policy_data # TODO this should ONLY be invoked by API Gateway
}
# Enable public access on endpoints Cloud Run service for rapidapi
resource "google_cloud_run_service_iam_policy" "realoptions_rapidapi_noauth" {
  location    = google_cloud_run_service.realoptions_rapidapi.location
  project     = google_cloud_run_service.realoptions_rapidapi.project
  service     = google_cloud_run_service.realoptions_rapidapi.name
  policy_data = data.google_iam_policy.noauth.policy_data
}
locals {
  realoptions_url = google_cloud_run_service.realoptions.status[0].url
}
output "realoptions_rapidapi_url" {
  value = replace(google_cloud_run_service.realoptions_rapidapi.status[0].url, "https://", "")
}

resource "google_api_gateway_api" "api_realoptions" {
  provider = google-beta
  api_id   = "api-realoptions"
}

resource "google_api_gateway_api_config" "api_realoptions" {
  provider      = google-beta
  api           = google_api_gateway_api.api_realoptions.api_id
  api_config_id = "cfg"

  openapi_documents {
    document {
      path = "spec.yaml"
      contents = base64encode(templatefile(
        "../docs/openapi_gcp.yml",
        {
          VERSION_MAJOR = var.version_major
          HOST          = local.realoptions_url
          PROJECT_ID    = var.project
        }
      ))
    }
  }
  lifecycle {
    create_before_destroy = true
  }
}

resource "google_cloud_run_domain_mapping" "domain_mapping" {
  name     = var.custom_gcp_domain
  location = var.region
  spec {
    route_name = google_cloud_run_service.realoptions.name
  }
  metadata {
    namespace = var.project
  }
  depends_on = [google_cloud_run_service.realoptions]
}

resource "google_cloud_run_domain_mapping" "domain_mapping_rapid_api" {
  name     = var.custom_rapid_api_domain
  location = var.region
  spec {
    route_name = google_cloud_run_service.realoptions_rapidapi.name
  }
  metadata {
    namespace = var.project
  }
  depends_on = [google_cloud_run_service.realoptions_rapidapi]
}