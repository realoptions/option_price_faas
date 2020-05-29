# Filename: main.tf# Configure GCP project
provider "google" {
  project = var.project
}

resource "google_project_service" "container_registry" {
  service    = "containerregistry.googleapis.com"
  disable_dependent_services = true
}

resource "google_project_service" "cloud_run" {
  service    = "run.googleapis.com"
}

# gateway container for auth handling
resource "google_cloud_run_service" "realoptions_gateway" {
  name     = "realoptions-gateway2"
  location = var.region
  project = var.project
  template {
    spec {
      containers {
        image = "gcr.io/endpoints-release/endpoints-runtime-serverless:2"
      }
    }
  }
  traffic {
    percent         = 100
    latest_revision = true
  }
  depends_on = [google_project_service.cloud_run]
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
resource "google_cloud_run_service_iam_policy" "noauth" {
  location    = google_cloud_run_service.realoptions_gateway.location
  project     = google_cloud_run_service.realoptions_gateway.project
  service     = google_cloud_run_service.realoptions_gateway.name
  policy_data = data.google_iam_policy.noauth.policy_data
}
locals {
  realoptions_gateway_url = google_cloud_run_service.realoptions_gateway.status[0].url
}
output "realoptions_gateway_url" {
  value = google_cloud_run_service.realoptions_gateway.status[0].url
}


resource "google_endpoints_service" "openapi_service" {
  service_name = replace(local.realoptions_gateway_url, "https://", "")
  project        = var.project
  openapi_config = templatefile(
    "../docs/openapi_v2.yml",
    {
      VERSION_MAJOR = var.api_version_major
      HOST = local.realoptions_gateway_url
      VISIBLE_HOST = replace(local.realoptions_gateway_url, "https://", "")
      PROJECT_ID = var.project
    }
  )
  depends_on = [google_cloud_run_service.realoptions_gateway]
  # Work-around for circular dependency between the Cloud Endpoints and ESP. See
  # https://github.com/terraform-providers/terraform-provider-google/issues/5528
  provisioner "local-exec" {
    command = "gcloud beta run services update ${google_cloud_run_service.realoptions_gateway.name} --set-env-vars ENDPOINTS_SERVICE_NAME=${self.service_name} --set-env-vars=ESPv2_ARGS=--cors_preset=basic --project ${var.project} --platform=managed --region=${var.region}"
  }
}

# actual app logic
resource "google_cloud_run_service" "realoptions" {
  name     = var.service_name
  location = var.region
  project = var.project
  template {
    spec {
      containers {
        image = "gcr.io/${var.project}/${var.service_name}:${var.github_sha}"
      }
    }
  }
  traffic {
    percent         = 100
    latest_revision = true
  }
  depends_on = [google_project_service.cloud_run]
}

resource "google_cloud_run_domain_mapping" "domain_mapping" {
  name = var.custom_api_domain
  location = var.region
  spec {
    route_name = google_cloud_run_service.realoptions.name
  }
  metadata {
    namespace = var.project
  }
  depends_on = [google_cloud_run_service.realoptions]
}




