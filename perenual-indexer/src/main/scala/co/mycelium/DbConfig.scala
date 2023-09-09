package co.mycelium

import zio.Config, Config._
import zio._
import zio.ConfigProvider

final case class DbConfig(
                           host: String,
                           port: Int,
                           username: String,
                           password: Secret,
                           database: String
                         )

object DbConfig {

  private val dbConfig: Config[DbConfig] =
    (
      string("PG_HOST").withDefault("localhost") zip
      int("PG_PORT").withDefault(5432) zip
      string("PG_USER").withDefault("mycelium") zip
      secret("PG_PASS").withDefault(Secret("mycelium")) zip
      string("PG_DB").withDefault("mycelium")
    ).map { case (host, port, user, pass, db) => DbConfig(host, port, user, pass, db)}

  val layer: ZLayer[Any, Error, DbConfig] = ZLayer(ConfigProvider.fromEnv().load(dbConfig))
}

