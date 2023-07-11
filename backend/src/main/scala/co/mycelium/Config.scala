package co.mycelium

import ciris._
import cats.implicits._

final case class DbConfig(host: String, port: Int, username: String, password: Secret[String], database: String)

final case class S3BlobConfig(host: String, accessKey: String, secretKey: Secret[String])

final case class AppConfig(db: DbConfig, blob: S3BlobConfig)

object AppConfig {

  private val dbConfig =
    (
      env("PG_HOST").as[String].default("localhost"),
      env("PG_PORT").as[Int].default(5432),
      env("PG_USER").as[String].default("mycelium"),
      env("PG_PASS").as[String].secret.default(Secret("mycelium")),
      env("PG_DB").as[String].default("mycelium")
    ).parMapN(DbConfig)

  private val blobConfig =
    (
      env("S3_HOST").as[String].default("http://127.0.0.1:9000"),
      env("S3_ACCESS_KEY").as[String].default("minio"),
      env("S3_SECRET_KEY").as[String].secret.default(Secret("miniominio"))
    ).parMapN(S3BlobConfig)

  val config: ConfigValue[Effect, AppConfig] = (dbConfig, blobConfig).parMapN(AppConfig.apply)
}