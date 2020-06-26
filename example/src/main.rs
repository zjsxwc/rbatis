#![allow(unused_imports)]
#![allow(unreachable_patterns)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_must_use)]
#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
use std::convert::Infallible;
use std::sync::{ Mutex};
use std::thread::sleep;
use std::time::{Duration};

use fast_log::log::RuntimeType;
use log::{info};
use serde_json::{json, Value};
use tide::Request;
use rbatis::rbatis::Rbatis;
use rbatis_core::db::DBPool;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

///数据库表模型,支持BigDecimal ,DateTime ,rust基本类型（int,float,uint,string,Vec,Array）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Activity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub pc_link: Option<String>,
    pub h5_link: Option<String>,
    pub pc_banner_img: Option<String>,
    pub h5_banner_img: Option<String>,
    pub sort: Option<String>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub create_time: Option<DateTime<chrono::Utc>>,
    pub version: Option<i32>,
    pub delete_flag: Option<i32>,
}

//示例 mysql 链接地址
pub const MYSQL_URL: &'static str = "mysql://root:123456@localhost:3306/test";

// 示例-Rbatis示例初始化
lazy_static! {
  static ref RB:Rbatis<'static>={
         let r=Rbatis::new();
         async_std::task::block_on(async{
           r.link(MYSQL_URL).await.unwrap();
         });
         return r;
  };
}

//初始化Tokio运行时
lazy_static! {
 static ref RT:Mutex<tokio::runtime::Runtime> = Mutex::new(tokio::runtime::Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap());
}


//启动web服务，并且对表执行 count统计
fn main() {
    async_std::task::block_on(async {
        fast_log::log::init_log("requests.log", &RuntimeType::Std).unwrap();
        let mut app = tide::new();
        app.at("/").get(|_: Request<()>| async move {
            // println!("accept req[{} /test] arg: {:?}",req.url().to_string(),a);
            let v = RB.fetch("", "SELECT count(1) FROM biz_activity;").await;
            if v.is_ok() {
                let data: Value = v.unwrap();
                Ok(data.to_string())
            } else {
                Ok(v.err().unwrap().to_string())
            }
        });
        //app.at("/").get(|_| async { Ok("Hello, world!") });
        let addr = "0.0.0.0:8000";
        println!("http server listen on {}", addr);
        app.listen(addr).await.unwrap();
    });
}


// 示例-打印日志
#[test]
pub fn test_log() {
    //1 启用日志(可选，不添加则不加载日志库)
    fast_log::log::init_log("requests.log", &RuntimeType::Std).unwrap();
    info!("print data");
    sleep(Duration::from_secs(1));
}

//示例-Rbatis直接使用驱动
#[test]
pub fn test_use_driver() {
    async_std::task::block_on(
        async move {
            fast_log::log::init_log("requests.log", &RuntimeType::Std).unwrap();
            let pool = DBPool::new(MYSQL_URL).await.unwrap();
            let mut conn = pool.acquire().await.unwrap();
            let mut c = conn.fetch("SELECT count(1) FROM biz_activity;").unwrap();
            let r: serde_json::Value = c.decode_json().await.unwrap();
            println!("done:{:?}", r);
        }
    );
}

//示例-Rbatis直接使用驱动-prepared stmt sql
#[test]
pub fn test_prepare_sql() {
  async_std::task::block_on(
        async move {
            fast_log::log::init_log("requests.log", &RuntimeType::Std).unwrap();
            let rb = Rbatis::new();
            rb.link(MYSQL_URL).await.unwrap();
            let arg = &vec![json!(1), json!("test%")];
            let r: Vec<Activity> = rb.fetch_prepare("", "SELECT * FROM biz_activity WHERE delete_flag =  ? AND name like ?", arg).await.unwrap();
            println!("done:{:?}", r);
        }
    );
}


//示例-Rbatis使用py风格的语法查询
#[test]
pub fn test_py_sql() {
    async_std::task::block_on(async move {
        fast_log::log::init_log("requests.log", &RuntimeType::Std).unwrap();
        let rb = Rbatis::new();
        rb.link(MYSQL_URL).await.unwrap();
        let py = r#"
    SELECT * FROM biz_activity
    WHERE delete_flag = #{delete_flag}
    if name != null:
      AND name like #{name+'%'}
    if ids != null:
      AND id in (
      trim ',':
         for item in ids:
           #{item},
      )"#;
        let data: serde_json::Value = rb.py_fetch("", py, &json!({   "delete_flag": 1 })).await.unwrap();
        println!("{}", data);
    });
}


//示例-Rbatis使用传统XML风格的语法查询
#[test]
pub fn test_xml_sql() {
    async_std::task::block_on(
        async move {
            let mut rb = Rbatis::new();
            rb.link(MYSQL_URL).await.unwrap();
            rb.load_xml("test", r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE mapper PUBLIC "-//mybatis.org//DTD Mapper 3.0//EN"
        "https://raw.githubusercontent.com/zhuxiujia/Rbatis/master/rbatis-mapper.dtd">
<mapper>
    <result_map id="BaseResultMap" table="biz_activity">
        <id column="id"/>
        <result column="name" lang_type="string"/>
        <result column="pc_link" lang_type="string"/>
        <result column="h5_link" lang_type="string"/>
        <result column="pc_banner_img" lang_type="string"/>
        <result column="h5_banner_img" lang_type="string"/>
        <result column="sort" lang_type="string"/>
        <result column="status" lang_type="number"/>
        <result column="remark" lang_type="string"/>
        <result column="version" lang_type="number" version_enable="true"/>
        <result column="create_time" lang_type="time"/>
        <result column="delete_flag" lang_type="number" logic_enable="true" logic_undelete="1"
                logic_deleted="0"/>
    </result_map>
    <select id="select_by_condition">
        <bind name="pattern" value="'%' + name + '%'"/>
        select * from biz_activity
        <where>
            <if test="name != null">and name like #{pattern}</if>
            <if test="startTime != null">and create_time >= #{startTime}</if>
            <if test="endTime != null">and create_time &lt;= #{endTime}</if>
        </where>
        order by create_time desc
        <if test="page != null and size != null">limit #{page}, #{size}</if>
    </select>
</mapper>"#).unwrap();
        }
    )
}

//示例-Rbatis使用事务
#[test]
pub fn test_tx() {
    async_std::task::block_on(async {
        let rb = Rbatis::new();
        rb.link(MYSQL_URL).await.unwrap();
        let tx_id = "1";
        rb.begin(tx_id).await.unwrap();
        let v: serde_json::Value = rb.fetch(tx_id, "SELECT count(1) FROM biz_activity;").await.unwrap();
        println!("{}", v.clone());
        rb.commit(tx_id).await.unwrap();
    });
}

/// 示例-Rbatis使用web框架Tide、async_std
#[test]
pub fn test_tide() {
    async_std::task::block_on(async {
        fast_log::log::init_log("requests.log", &RuntimeType::Std).unwrap();
        let mut app = tide::new();
        app.at("/").get(|_: Request<()>| async move {
            // println!("accept req[{} /test] arg: {:?}",req.url().to_string(),a);
            let v = RB.fetch("", "SELECT count(1) FROM biz_activity;").await;
            if v.is_ok() {
                let data: Value = v.unwrap();
                Ok(data.to_string())
            } else {
                Ok(v.err().unwrap().to_string())
            }
        });
        //app.at("/").get(|_| async { Ok("Hello, world!") });
        let addr = "0.0.0.0:8000";
        println!("server on {}", addr);
        app.listen(addr).await.unwrap();
    });
}


async fn hello(_: hyper::Request<hyper::Body>) -> Result<hyper::Response<hyper::Body>, Infallible> {
    let v = RB.fetch("", "SELECT count(1) FROM biz_activity;").await;
    if v.is_ok() {
        let data: Value = v.unwrap();
        Ok(hyper::Response::new(hyper::Body::from(data.to_string())))
    } else {
        Ok(hyper::Response::new(hyper::Body::from(v.err().unwrap().to_string())))
    }
}
// 示例-Rbatis使用web框架hyper/Tokio
#[test]
pub fn test_hyper() {
    RT.lock().unwrap().block_on(async {
        RB.link(MYSQL_URL).await.unwrap();
        fast_log::log::init_log("requests.log", &RuntimeType::Std).unwrap();
        let make_svc = hyper::service::make_service_fn(|_conn| {
            async { Ok::<_, Infallible>(hyper::service::service_fn( hello)) }
        });
        let addr = ([0, 0, 0, 0], 8000).into();
        let server = hyper::Server::bind(&addr).serve(make_svc);
        println!("Listening on http://{}", addr);
        server.await.unwrap();
    });
}