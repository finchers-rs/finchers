require 'rake'

task :test do
    sh "cargo clippy --tests"
    sh "cargo test"
    sh "cargo test --all-features"
    sh "cargo test --no-default-features"
    sh "cargo test -p doctest"
end

task :install_hooks do
    sh "cargo clean -p cargo-husky"
    sh "cargo check -p cargo-husky"
end

task default: :test
