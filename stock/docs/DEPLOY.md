# 部署指南

## 环境要求

| 组件 | 最低版本 | 说明                    |
|------|----------|-------------------------|
| Rust | 1.84     | 原生部署时需要           |
| Docker| 24+     | Docker 部署时需要（推荐）|
| 网络 | 可访问外网 | 用于定时抓取商品价格数据 |

该应用是一个独立的 **Linux x86_64** 二进制程序，内部自带 Web 服务器（Axum），不需要 Nginx 反向代理也能直接运行（建议生产环境在前面加一层 Nginx 做 TLS 终止）。

---

## 一、Docker 部署（推荐）

### 1. 将项目上传到服务器

```bash
# 在本地打包
git clone <你的仓库地址> /opt/stock
cd /opt/stock

# 或使用 scp / rsync 直接上传项目目录
```

### 2. 构建镜像

```bash
cd /opt/stock
docker build -t stock:latest .
```

首次构建会下载 Rust 依赖，耗时约 **3-8 分钟**（视网络状况而定）。后续代码修改后重新构建，由于 Docker 层缓存，通常只需要 **30-60 秒**。

### 3. 运行容器

```bash
docker run -d \
  --name stock \
  --restart unless-stopped \
  -p 30001:30001 \
  -v /opt/stock/data:/app/data \
  stock:latest
```

参数说明：
- `--restart unless-stopped` — 服务器重启后自动启动容器
- `-p 30001:30001` — 将主机的 30001 端口映射到容器内
- `-v /opt/stock/data:/app/data` — 数据持久化，即使容器删除数据也不会丢失

### 4. 验证

```bash
# 查看容器状态
docker ps

# 查看日志
docker logs -f stock

# 测试访问
curl http://localhost:30001
```

### 5. 更新

```bash
cd /opt/stock
git pull                    # 拉取最新代码
docker build -t stock:latest .   # 重新构建
docker stop stock
docker rm stock
docker run -d --name stock \
  --restart unless-stopped \
  -p 3001:3001 \
  -v /opt/stock/data:/app/data \
  stock:latest
```

---

## 二、原生部署（直接运行二进制）

### 1. 在服务器上安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### 2. 编译（两种方式）

**方式 A：在服务器上直接编译**

```bash
cd /opt/stock
cargo build --release
```

**方式 B：在本地交叉编译后上传（推荐，避免在服务器上装 Rust）**

在本地（Windows 或 macOS）：
```bash
# 如果本地安装了 Linux 目标
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```

然后将 `target/x86_64-unknown-linux-gnu/release/stock` 上传到服务器 `/opt/stock/`。

### 3. 创建运行用户和目录

```bash
# 创建专用用户
sudo useradd -r -s /usr/sbin/nologin -m -d /opt/stock stock

# 创建项目目录结构
sudo mkdir -p /opt/stock/data /opt/stock/templates /opt/stock/static

# 复制编译产物
sudo cp target/release/stock /opt/stock/
sudo cp -r templates/ /opt/stock/
sudo cp -r static/ /opt/stock/

# 设置权限
sudo chown -R stock:stock /opt/stock
```

### 4. 配置 systemd 服务

```bash
sudo cp stock.service /etc/systemd/system/stock.service
sudo systemctl daemon-reload
sudo systemctl enable stock
sudo systemctl start stock
```

### 5. 验证

```bash
sudo systemctl status stock
sudo journalctl -u stock -f
curl http://localhost:3001
```

### 6. 更新

```bash
cd /opt/stock
git pull
cargo build --release
sudo systemctl restart stock
```

---

## 三、Nginx 反向代理（可选，生产环境建议）

`ssl_certificate` 不要指向 `*_chain.pem`，要指向 `server.pem + chain.pem` 合成后的 fullchain 文件

先在服务器执行：

```bash
cd /etc/letsencrypt/live/v.qianlima.site

sudo cat scs1782638325334_v.qianlima.site_server.pem \
  scs1782638325334_v.qianlima.site_chain.pem \
  | sudo tee scs1782638325334_v.qianlima.site_fullchain.pem > /dev/null
```

然后 nginx 配置改成这样：

```nginx
server {
    listen 443 ssl;
    server_name v.qianlima.site;

    ssl_certificate /etc/letsencrypt/live/v.qianlima.site/scs1782638325334_v.qianlima.site_fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/v.qianlima.site/scs1782638325334_v.qianlima.site_private_key.pem;

    include /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

    location / {
        proxy_pass http://localhost:30001;
        proxy_http_version 1.1;

        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

如果还需要 HTTP 自动跳 HTTPS，再加一个 80 端口配置：

```nginx
server {
    listen 80;
    server_name v.qianlima.site;

    return 301 https://$host$request_uri;
}
```

最后执行：

```bash
sudo nginx -t
sudo systemctl reload nginx
```


---

## 四、防火墙和端口

| 端口 | 用途            | 建议                           |
|------|-----------------|--------------------------------|
| 3001 | 应用 HTTP 服务  | 使用 Nginx 反向代理后可对内网开放 |
| 80   | HTTP            | 如需公网访问需开放               |
| 443  | HTTPS           | 如需公网访问需开放               |

如果直接暴露 3001 端口：

```bash
# Ubuntu/Debian
sudo ufw allow 3001/tcp

# CentOS/RHEL
sudo firewall-cmd --add-port=3001/tcp --permanent
sudo firewall-cmd --reload
```

---

## 五、数据持久化说明

价格数据以 JSON 格式存储在以下结构：

```
data/commodities/
├── live_pig/prices.json
├── corn/prices.json
├── soybean_meal/prices.json
├── titanium_dioxide/prices.json
└── sulfur/prices.json
```

- Docker 部署：通过 `-v` 卷挂载将 `data/` 目录映射到宿主机
- 原生部署：直接存储在 `/opt/stock/data/`

如果删除容器或重新部署，务必保证 `data/` 目录的数据不丢失，否则所有历史价格数据将丢失。

---

## 六、日志查看

| 部署方式 | 命令                |
|----------|---------------------|
| Docker   | `docker logs -f stock` |
| systemd  | `journalctl -u stock -f` |

---

## 七、常见问题

### Q: 编译失败，提示 OpenSSL 相关错误

安装系统依赖：
```bash
sudo apt update
sudo apt install pkg-config libssl-dev
```

### Q: 无法访问外网，数据抓取失败

中国境内的服务器可能需要配置 HTTP 代理：
```rust
// 在 main.rs 或环境变量中设置
std::env::set_var("HTTP_PROXY", "http://your-proxy:port");
std::env::set_var("HTTPS_PROXY", "http://your-proxy:port");
```

### Q: 时区不对，计划任务执行时间与预期不符

设置服务器时区：
```bash
sudo timedatectl set-timezone Asia/Shanghai
```

### Q: 端口被占用

修改 `main.rs` 中的端口号，或在 Docker 运行时映射到不同端口：
```bash
docker run -p 8080:30001 ...
```
