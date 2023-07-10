package co.mycelium

import cats.data.Kleisli
import cats.implicits._
import cats.effect._
import co.mycelium.db.Repositories
import co.mycelium.endpoints.Stations
import com.comcast.ip4s._
import org.http4s.{HttpApp, Request, Response}
import org.http4s.ember.server.EmberServerBuilder
import org.http4s.server.middleware.{ErrorAction, ErrorHandling}
import org.http4s.server.{Router, Server}
import org.http4s.server.staticcontent._
import org.typelevel.log4cats.LoggerFactory
import org.typelevel.log4cats.slf4j.Slf4jFactory

object Main extends IOApp {

  implicit val loggerFactory: LoggerFactory[IO] = Slf4jFactory.create[IO]

  override def run(args: List[String]): IO[ExitCode] =
    app.use(_ => IO.never).as(ExitCode.Success)

  def httpApp(repositories: Repositories[IO]): HttpApp[IO] = {

    val api = Router(
      "api" -> Stations.routes(repositories)
    )
    val files = fileService[IO](FileService.Config("."))

    (api <+> files).orNotFound
  }

  private def errorHandling(route: Kleisli[IO, Request[IO], Response[IO]]) = ErrorHandling.Recover.total(
    ErrorAction.log(
      route,
      messageFailureLogAction = (t, msg) =>
        IO.println(msg) >>
          IO.println(t),
      serviceErrorLogAction = (t, msg) =>
        IO.println(msg) >>
          IO.println(t)
    )
  )

  val app: Resource[IO, Server] =
    for {
      tx <- Transactors.pg[IO](DbConfig("localhost", 5432, "mycelium", "mycelium", "mycelium"))
      repos = Repositories.fromTransactor(tx)
      server <- EmberServerBuilder
        .default[IO]
        .withHost(ipv4"0.0.0.0")
        .withPort(port"8080")
        .withHttpApp(errorHandling(httpApp(repos)))
        .build
    } yield server
}
