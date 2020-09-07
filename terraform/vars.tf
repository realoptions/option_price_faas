variable "project" {}
variable "region" {}
variable "service_name" {}
variable "service_name_auth" {}
variable "github_sha" {}
variable "gcp_api_version" {}
variable "rapid_api_version" {}
variable "custom_rapid_api_domain" {
  description="name of hostname/domain name for API"
}
variable "custom_gcp_domain" {
  description="name of hostname/domain name for API"
}