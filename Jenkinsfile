pipeline {
  agent none
  environment {
    CARGO_HOME = "/.cargo"
    RUST_VERSION = "1.57"
    RUST_IMAGE = "rust:${RUST_VERSION}-slim-bullseye"
  }
  options {
    timeout(time: 1, unit: "HOURS")
    timestamps()
  }
  stages {
    stage("Test") {
      agent {
        docker {
          image RUST_IMAGE
          args "-v /jenkins-cache/cargo:${CARGO_HOME}"
        }
      }
      steps {
        sh "cargo test --locked"
      }
      post {
        cleanup {
          cleanWs()
        }
      }
    }
    stage("Build (Linux)") {
      agent {
        docker {
          image RUST_IMAGE
          args "-v /jenkins-cache/cargo:${CARGO_HOME}"
        }
      }
      steps {
        sh "cargo build --bins --release"
      }
      post {
        success {
          archiveArtifacts(artifacts: "target/release/pjsh", fingerprint: true)
        }
        cleanup {
          cleanWs()
        }
      }
    }
  }
}
