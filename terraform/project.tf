# Filename: main.tf# Configure GCP project
provider "google" {
  project = var.project
}

terraform {
  backend "gcs" {
    bucket  = "artifacts.finside.appspot.com"
    prefix  = "terraform/state"
  }
}

data "terraform_remote_state" "tfstate" {
  backend = "gcs"
  config = {
    bucket  = "artifacts.finside.appspot.com"
    prefix  = "prod"
  }
}

resource "google_project_service" "container_registry" {
  service    = "containerregistry.googleapis.com"
  disable_dependent_services = true
}

resource "google_project_service" "cloud_run" {
  service    = "run.googleapis.com"
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

# gateway container for auth handling
resource "google_cloud_run_service" "realoptions_gateway" {
  name     = "${var.service_name}-gateway"
  location = var.region
  project = var.project
  template {
    spec {
      containers {
        image = "gcr.io/endpoints-release/endpoints-runtime-serverless:2"
        env {
          name = "ESPv2_ARGS"
          value = "--cors_preset=basic"
        }
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
  realoptions_gateway_url = replace(google_cloud_run_service.realoptions_gateway.status[0].url, "https://", "")
}
locals {
  realoptions_url = replace(google_cloud_run_service.realoptions.status[0].url, "https://", "")
}
output "realoptions_gateway_url" {
  value = replace(google_cloud_run_service.realoptions_gateway.status[0].url, "https://", "")
}


resource "google_endpoints_service" "openapi_service" {
  service_name = local.realoptions_gateway_url
  project        = var.project
  openapi_config = templatefile(
    "../docs/openapi_v2.yml",
    {
      VERSION_MAJOR = var.api_version_major
      HOST = local.realoptions_url
      VISIBLE_HOST = local.realoptions_gateway_url
      PROJECT_ID = var.project
    }
  )
  depends_on = [google_cloud_run_service.realoptions_gateway]
  # Work-around for circular dependency between the Cloud Endpoints and ESP. See
  # https://github.com/terraform-providers/terraform-provider-google/issues/5528
  provisioner "local-exec" {
    # https://cloud.google.com/endpoints/docs/openapi/get-started-cloud-run
    command = "gcloud beta run services update ${google_cloud_run_service.realoptions_gateway.name} --update-env-vars ENDPOINTS_SERVICE_NAME=${local.realoptions_gateway_url} --project ${var.project} --platform=managed --region=${var.region}"
  }
}



resource "google_cloud_run_domain_mapping" "domain_mapping" {
  name = var.custom_api_domain
  location = var.region
  spec {
    route_name = google_cloud_run_service.realoptions_gateway.name
  }
  metadata {
    namespace = var.project
  }
  depends_on = [google_cloud_run_service.realoptions_gateway]
}