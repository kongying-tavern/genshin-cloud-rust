# 空荧酒馆·原神地图 Rust 后端

## 如何构建

To ensure the environment is unified, this project has been deployed automatically using Docker Compose. Make sure Docker and Cargo are installed on the machine.

Before the formal deployment, you should create a file named `.env` in the project folder, which stores the configuration information of the database that the server will connect to. The example content is as follows:

```env
MARIADB_ROOT_PASSWORD=root
DB_PASSWORD=root
```

Please make sure that the configuration file has correctly filled in all the keys involved in the above example, and cannot be omitted.

Before starting, please make sure that `cargo-make` is installed:

```bash
cargo install --force cargo-make
```

Then execute the following command in the project directory to build the cluster:

```bash
cargo make build
```

If you need to debug in real time, start the cluster in this way:

```bash
cargo make dev
```

While deploying to the server, there are some differences in the way the cluster starts:

```bash
cargo make -p production -e DB_USERNAME=<username> -e DB_PASSWORD=<password> up
# Or use the .env file to keep the password on the server,
# ensuring that the password itself does not appear in the terminal execution history
cargo make -p production --env-file=<filepath> up
```

## 逻辑分块

主要分为3大块：

1. 业务模块
2. 权限模块
3. 功能模块

### 业务模块

业务模块为服务于地图逻辑的模块

当前提供的地图逻辑实体有：

1. 地区
2. 物品
3. 点位
4. 图标
5. 路线

其中，对以下逻辑实体提供打点-审核机制：

1. 点位
2. 路线（TODO）

对以下逻辑实体提供分类支持：

1. 物品
2. 图标

以下的数据结构为树状结构：

1. 所有的分类
2. 地区

### 权限模块

当前权限系统为单角色，共分为以下几个等级（权限由高到低）

1. 系统管理员——最高权限，管理权限模块
2. 地图管理员——管理业务模块
3. 审核员——拥有审核点位的权限
4. 测试服打点员——拥有对测试服数据的访问权限，经管理员授权可以绕过审核机制
5. 正式服打点员——拥有提交打点的权限
6. 后台准用户——无权限
7. 前台用户——匿名用户
