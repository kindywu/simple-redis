# 初始化

- cargo generate tyr-rust-bootcamp/template
- update master -> main
- update cliff.toml postprocessors.replace = https://github.com/kindywu/03-simple-redis
- pre-commit install
- git remote add origin https://github.com/kindywu/03-simple-redis.git

# 设置日志级别(windows)

- $env:RUST_LOG="info"

# Git commit

- git status
- git add .
- git commit -a -m "feat: first thread example"
- git commit -a --amend
- git push

- git tag -a v2-thread
- git push origin v2-thread

- git tag
- git checkout v2-3-metrics-1
