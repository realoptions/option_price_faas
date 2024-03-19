# Filename: main.tf# Configure GCP project
provider "google" {
  project = var.project
}

terraform {
  backend "gcs" {
    bucket = "artifacts.finside.appspot.com"
    prefix = "terraform/setup_state"
  }
}

resource "google_project_service" "container_registry" {
  service                    = "containerregistry.googleapis.com"
  disable_dependent_services = true
}

resource "google_project_service" "artifact_registry" {
  service                    = "artifactregistry.googleapis.com"
  disable_dependent_services = true
}

resource "google_project_service" "cloud_run" {
  service = "run.googleapis.com"
}

/// Artifact registry and container registry already manually created