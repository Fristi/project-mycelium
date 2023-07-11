package co.mycelium

import cats.effect.{Async, Resource}
import com.zaxxer.hikari.HikariConfig
import doobie.hikari.HikariTransactor
import org.flywaydb.core.Flyway

import javax.sql.DataSource
object Transactors {
  def pg[F[_] : Async](cfg: DbConfig): Resource[F, HikariTransactor[F]] = {
    def flyway(ds: DataSource) =
      Async[F].delay(Flyway.configure().locations("migrations").dataSource(ds).load().migrate())

    val config = new HikariConfig()
    config.setPoolName("mycelium")
    config.setJdbcUrl(s"jdbc:postgresql://${cfg.host}:${cfg.port}/${cfg.database}")
    config.setUsername(cfg.username)
    config.setPassword(cfg.password.value)
    config.setValidationTimeout(1000)
    config.setConnectionTimeout(2000)
    config.setDriverClassName("org.postgresql.Driver")
    config.setMaximumPoolSize(10)

    for {
      tx <- HikariTransactor.fromHikariConfig(config)
      _ <- Resource.eval(tx.configure(flyway))
    } yield tx
  }

}