variable "project" {}
variable "user" {}
variable "region" {}

variable "billing_account" {
  description = "Billing account display name"
}

variable "repository_name" {
  default = "wild-workouts"
}

variable "custom_api_domain" {
  description="name of hostname/domain name for API"
}
