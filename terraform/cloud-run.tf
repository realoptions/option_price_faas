module cloud_run_trainings_http {
  source = "./service"

  project    = var.project
  location   = var.region
  dependency = null_resource.init_docker_images

  name     = "trainings"
  protocol = "http"
  auth     = false

  #envs = [
  #  {
  #    name  = "TRAINER_GRPC_ADDR"
  #    value = module.cloud_run_trainer_grpc.endpoint
  #  },
  #  {
  ##    name  = "USERS_GRPC_ADDR"
  #   value = module.cloud_run_users_grpc.endpoint
  #  }
  #]
}
