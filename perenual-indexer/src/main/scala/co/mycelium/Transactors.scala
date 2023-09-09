package co.mycelium

import com.zaxxer.hikari.HikariConfig
import doobie.hikari.HikariTransactor
import doobie.util.transactor.Transactor
import zio.interop.catz._
import zio.interop.catz.implicits._
import zio.{Task, ZIO, ZLayer}
object Transactors {
  def pg(cfg: DbConfig) = {
    val config = new HikariConfig()
    val pwd = cfg.password.value.mkString("")
    config.setPoolName("mycelium")
    config.setJdbcUrl(s"jdbc:postgresql://${cfg.host}:${cfg.port}/${cfg.database}")
    config.setUsername(cfg.username)
    config.setPassword(pwd)
    config.setValidationTimeout(1000)
    config.setConnectionTimeout(2000)
    config.setDriverClassName("org.postgresql.Driver")
    config.setMaximumPoolSize(10)

    HikariTransactor.fromHikariConfig(config).toScopedZIO
  }

  val layer: ZLayer[DbConfig, Throwable, Transactor[Task]] =
    ZLayer.scoped {
      for {
        config <- ZIO.service[DbConfig]
        tx <- pg(config)
      } yield tx
    }

}

