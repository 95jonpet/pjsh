pipeline {
  agent none
  environment {
    BASE_VERSION ="0.1.0"
    VERSION = "${env.BASE_VERSION}${env.BRANCH_NAME == 'main' ? '-SNAPSHOT' : "~${env.BRANCH_NAME}"}"
    CARGO_HOME = "/.cargo"
  }
  options {
    timeout(time: 1, unit: "HOURS")
    timestamps()
  }
  stages {
    stage("Build & Test") {
      parallel {
        stage("Test") {
          agent {
            dockerfile {
              args "-v /jenkins-cache/cargo:${CARGO_HOME}"
            }
          }
          steps {
            sh label: "Run tests", script: "cargo test --locked"
            sh label: "Run linter", script: "cargo clippy -- -D warnings"
            sh label: "Run rustfmt", script: "scripts/check-code-format.sh"
          }
          post {
            cleanup {
              cleanWs()
            }
          }
        }
        stage("Build (Linux)") {
          agent {
            dockerfile {
              args "-v /jenkins-cache/cargo:${CARGO_HOME}"
            }
          }
          steps {
            sh label: "Build release binary", script: "cargo build --bins --release"
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
      }
    }
    stage("Package (Linux)") {
      agent {
        label "docker"
      }
      steps {
        unstash(name: "linux-binary")
        sh label: "Build Linux packages", script: """
          build/build-linux-packages.sh \
            '${VERSION}' \
            '${BUILD_NUMBER}' \
            build/package \
            target/release \
            target/package
        """
        sh label: "Verify Linux packages", script: "build/verify-linux-packages.sh target/package examples"
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
