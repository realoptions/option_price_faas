# Filename: main.tf# Configure GCP project
provider "google" {
  project = var.project
}

resource "google_project_service" "container_registry" {
  service    = "containerregistry.googleapis.com"
  #depends_on = [google_project.project]

  disable_dependent_services = true
}

resource "google_endpoints_service" "openapi_service" {
  service_name = replace(local.cloud_run_url, "https://", "")
  project        = var.project
  openapi_config = file("../docs/urlsubstitute.yml")
  # depends_on = [google_cloud_run_service.realoptions-gateway]
}
locals {
  service_for_gateway_docker = google_endpoints_service.openapi_service
}
#locals {
#  service_name = "${var.endpoint_service_name}.endpoints.${var.project_id}.cloud.goog"
#}

resource "google_project_service" "cloud_run" {
  service    = "run.googleapis.com"
  depends_on = [google_endpoints_service.openapi_service]
}



# gateway container for auth handling
resource "google_cloud_run_service" "realoptions-gateway" {
  name     = "realoptions-gateway"
  location = var.region
  project = var.project
  template {
    spec {
      containers {
        image = "gcr.io/${var.project}/endpoints-runtime-serverless:${var.github_sha}"
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
  depends_on = [google_project.cloud_run]
}


# Create public access
data "google_iam_policy" "noauth" {
  binding {
    role = "roles/run.invoker"
    members = [
      "allUsers",
    ]
  }
}
# Enable public access on Cloud Run service
resource "google_cloud_run_service_iam_policy" "noauth" {
  location    = google_cloud_run_service.realoptions-gateway.location
  project     = google_cloud_run_service.realoptions-gateway.project
  service     = google_cloud_run_service.realoptions-gateway.name
  policy_data = data.google_iam_policy.noauth.policy_data
}




# actual logic
resource "google_cloud_run_service" "realoptions" {
  name     = "realoptions"
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
  depends_on = [google_project.cloud_run]
}




resource "google_app_engine_domain_mapping" "domain_mapping" {
  domain_name = var.custom_api_domain

  ssl_settings {
    ssl_management_type = "AUTOMATIC"
  }
}




