pipeline {
  agent none
  environment {
    BASE_VERSION ="0.1.0"
    VERSION = "${env.BASE_VERSION}${env.BRANCH_NAME == 'main' ? '-SNAPSHOT' : "~${env.BRANCH_NAME}"}"
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
          stash(name: "linux-binary", includes: "target/release/pjsh")
        }
        cleanup {
          cleanWs()
        }
      }
    }
    stage("Package (Linux)") {
      agent {
        label "docker"
      }
      steps {
        unstash(name: "linux-binary")
        sh "build/build-linux-packages.sh '${VERSION}' '${BUILD_NUMBER}'"
      }
      post {
        success {
          archiveArtifacts(artifacts: "target/package/*", fingerprint: true)
        }
        cleanup {
          cleanWs()
        }
      }
    }
  }
}
