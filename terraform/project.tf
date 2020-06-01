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

resource "google_endpoints_service" "openapi_service" {
  service_name = replace(var.gateway_url, "https://", "")
  project        = var.project
  openapi_config = templatefile(
    "../docs/openapi_v2.yml",
    {
      VERSION_MAJOR = var.api_version_major
      HOST = var.gateway_url
      VISIBLE_HOST = replace(var.gateway_url, "https://", "")
      PROJECT_ID = var.project
    }
  )
  # depends_on = [google_cloud_run_service.realoptions_gateway]
  # Work-around for circular dependency between the Cloud Endpoints and ESP. See
  # https://github.com/terraform-providers/terraform-provider-google/issues/5528
  # have to redeploy gateway docker https://realoptions2-gateway-lnmfgwrxtq-uc.a.run.app  https://realoptions2-lnmfgwrxtq-uc.a.run.app
  # the bash script builds the docker image and redeploys
  provisioner "local-exec" {
    command = "./build_esp.sh"
    environment = {
      GATEWAY_SERVICE = self.service_name
      PROJECT_ID = var.project
      GITHUB_SHA = var.github_sha
      CLOUD_RUN_SERVICE = var.gateway_name
      RUN_REGION = var.region
    }  
  }

}

# gateway container for auth handling, dummy to get api url
resource "google_cloud_run_service" "realoptions_gateway" {
  name     = var.gateway_name
  location = var.region
  project = var.project
  template {
    spec {
      containers {
        image = "gcr.io/${var.project}/${var.gateway_name}:${var.github_sha}"
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
  autogenerate_revision_name=true
  depends_on = [google_endpoints_service.openapi_service]
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
  autogenerate_revision_name=true
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