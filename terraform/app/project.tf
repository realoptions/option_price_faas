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
}


# gateway container for auth handling
resource "google_cloud_run_service" "realoptions_gateway" {
  name     = "${var.service_name}-gateway"
  location = var.region
  project  = var.project
  template {
    spec {
      containers {
        image = "gcr.io/endpoints-release/endpoints-runtime-serverless:2"
        env {
          name  = "ESPv2_ARGS"
          value = "--cors_preset=basic"
        }

      }
    }
  }
  traffic {
    percent         = 100
    latest_revision = true
  }
  autogenerate_revision_name = true
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

# Enable public access on endpoints Cloud Run service for rapidapi
resource "google_cloud_run_service_iam_policy" "realoptions_rapidapi_noauth" {
  location    = google_cloud_run_service.realoptions_rapidapi.location
  project     = google_cloud_run_service.realoptions_rapidapi.project
  service     = google_cloud_run_service.realoptions_rapidapi.name
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
output "realoptions_rapidapi_url" {
  value = replace(google_cloud_run_service.realoptions_rapidapi.status[0].url, "https://", "")
}


resource "google_endpoints_service" "openapi_service" {
  service_name = local.realoptions_gateway_url
  project      = var.project
  openapi_config = templatefile(
    "../docs/openapi_gcp.yml",
    {
      VERSION_MAJOR = var.version_major
      HOST          = local.realoptions_url
      VISIBLE_HOST  = local.realoptions_gateway_url
      PROJECT_ID    = var.project
    }
  )
  depends_on = [google_cloud_run_service.realoptions_gateway]

}
resource "null_resource" "cloud_run_workaround" {
  # Work-around for circular dependency between the Cloud Endpoints and ESP. See
  # https://github.com/terraform-providers/terraform-provider-google/issues/5528
  provisioner "local-exec" {
    # https://cloud.google.com/endpoints/docs/openapi/get-started-cloud-run
    command = "gcloud beta run services update ${google_cloud_run_service.realoptions_gateway.name} --update-env-vars ENDPOINTS_SERVICE_NAME=${local.realoptions_gateway_url} --project ${var.project} --platform=managed --region=${var.region}"
  }
  # invalidate state, so this always runs 
  triggers = {
    always_run = "${timestamp()}"
  }
  depends_on = [google_endpoints_service.openapi_service]
}


resource "google_cloud_run_domain_mapping" "domain_mapping" {
  name     = var.custom_gcp_domain
  location = var.region
  spec {
    route_name = google_cloud_run_service.realoptions_gateway.name
  }
  metadata {
    namespace = var.project
  }
  depends_on = [google_cloud_run_service.realoptions_gateway]
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