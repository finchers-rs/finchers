require 'rake'

task :ci_flow do
    sh "cargo build --all"
    sh "env FINCHERS_DENY_WARNINGS=1 cargo test --features use-handlebars"
    sh "env FINCHERS_DENY_WARNINGS=1 cargo test --features use-tera"
    sh "env FINCHERS_DENY_WARNINGS=1 cargo test --features use-askama"
    sh "env FINCHERS_DENY_WARNINGS=1 cargo test --features use-horrorshow"
    sh "env FINCHERS_DENY_WARNINGS=1 cargo clippy"
end

task :install_hooks do
    sh "cargo clean -p cargo-husky"
    sh "cargo check -p cargo-husky"
end

task default: :ci_flow
