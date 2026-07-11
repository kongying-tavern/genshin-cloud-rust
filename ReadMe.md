# 空荧酒馆·原神地图 Rust 后端 / Genshin Map Cloud Rust Backend

[![Rust](https://github.com/kongying-tavern/genshin-cloud-rust/workflows/Rust/badge.svg)](https://github.com/kongying-tavern/genshin-cloud-rust/actions/workflows/rust.yml)

## 如何构建 / How to build

为了确保环境的一致性，本项目采用 Docker Compose 进行自动化部署。请确保机器上已安装 Docker 和 Cargo。

To ensure the environment is unified, this project has been deployed automatically using Docker Compose. Make sure Docker and Cargo are installed on the machine.

在部署之前，您应该在项目文件夹中创建一个名为 `.env` 的文件，其中存储服务器将连接到的数据库的配置信息。示例内容如下：

Before the formal deployment, you should create a file named `.env` in the project folder, which stores the configuration information of the database that the server will connect to. The example content is as follows:

```env
DB_PASSWORD=<password>
```

请确保以上的配置项已正确填写。

Please make sure that the configuration file has correctly filled in all the keys involved in the above example, and cannot be omitted.

正式开始前，请确保已安装 `cargo-make` 与 `docker`；如果需要本地调试，还需要安装 `docker-compose`。

Before starting, please make sure that `cargo-make` and `docker` are installed. If you need local debugging, you also need to install `docker-compose`.

之后，执行以下命令以构建集群：

Then execute the following command in the project directory to build the cluster:

```bash
cargo make build
```

如果您需要实时调试，可以通过以下方式启动集群：

If you need to debug in real time, start the cluster in this way:

```bash
cargo make dev
```
