package co.mycelium

import cats.Applicative
import cats.effect.{Async, IO, Resource}
import com.zaxxer.hikari.HikariConfig
import doobie.hikari.HikariTransactor
import org.flywaydb.core.Flyway
import retry._
import retry.RetryPolicies._

import javax.sql.DataSource
import scala.concurrent.duration.DurationInt
object Transactors {
  def pg[F[_] : Async](cfg: DbConfig): Resource[F, HikariTransactor[F]] = {
    def flyway(ds: DataSource) =
      Async[F].delay {
        Flyway
          .configure()
          .locations("migrations")
          .dataSource(ds)
          .load()
          .migrate()
      }

    type ResourceM[A] = Resource[F, A]

    def policy[F[_] : Applicative] =
      limitRetries[F](10) join exponentialBackoff[F](200.milliseconds)

    def handleError(error: Throwable, retryDetails: RetryDetails): Resource[F, Unit] = Resource.unit

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
      tx <- retryingOnAllErrors[HikariTransactor[F]].apply[ResourceM, Throwable](policy, handleError)(HikariTransactor.fromHikariConfig(config))
      _ <- Resource.eval(tx.configure(flyway))
    } yield tx
  }

}