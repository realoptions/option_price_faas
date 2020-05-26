
variable project {}
variable name {}
variable location {}

variable envs {
  type = list(object({
    name  = string
    value = string
  }))
  default = []
}
variable auth {
  type    = bool
  default = true
}
variable dependency {
  type = any
}


