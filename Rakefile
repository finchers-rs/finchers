require 'rake'

task :format_check do
    sh "cargo fmt -- --check"
end

task :ci_flow do
    sh "cargo update"
    sh "cargo fmt -- --check"
    sh "env FINCHERS_DENY_WARNINGS=1 cargo clippy --tests"
    sh "env FINCHERS_DENY_WARNINGS=1 cargo test"
    sh "env FINCHERS_DENY_WARNINGS=1 cargo test --all-features"
    sh "env FINCHERS_DENY_WARNINGS=1 cargo test --no-default-features"
    sh "cargo test -p doctest"
end

task :install_hooks do
    sh "cargo clean -p cargo-husky"
    sh "cargo check -p cargo-husky"
end
