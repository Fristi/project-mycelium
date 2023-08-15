package co.mycelium

import blobstore.s3.S3Store
import cats.data.Kleisli
import cats.effect._
import cats.implicits._
import co.mycelium.db.Repositories
import co.mycelium.endpoints.{Avatar, Stations}
import com.comcast.ip4s._
import org.http4s.ember.server.EmberServerBuilder
import org.http4s.server.{Router, Server}
import org.http4s.server.middleware.{CORS, CORSConfig, ErrorAction, ErrorHandling}
import org.http4s.server.staticcontent._
import org.http4s.{HttpApp, Request, Response}
import org.typelevel.log4cats.LoggerFactory
import org.typelevel.log4cats.slf4j.Slf4jFactory
import software.amazon.awssdk.auth.credentials.{AwsBasicCredentials, StaticCredentialsProvider}
import software.amazon.awssdk.core.client.config.ClientOverrideConfiguration
import software.amazon.awssdk.core.retry.RetryPolicy
import software.amazon.awssdk.core.retry.conditions.RetryCondition
import software.amazon.awssdk.http.async.SdkAsyncHttpClient
import software.amazon.awssdk.http.nio.netty.NettyNioAsyncHttpClient
import software.amazon.awssdk.regions.Region
import software.amazon.awssdk.services.s3.S3AsyncClient
import software.amazon.awssdk.services.s3.model.{BucketAlreadyExistsException, CreateBucketRequest}

import java.net.URI
import java.time.Duration

object Main extends IOApp {

  implicit val loggerFactory: LoggerFactory[IO] = Slf4jFactory.create[IO]

  override def run(args: List[String]): IO[ExitCode] =
    app.use(_ => IO.never).as(ExitCode.Success)

  def httpApp(repositories: Repositories[IO]): HttpApp[IO] = {

    val server = Router(
      "api"    -> Stations.routes(repositories),
      "avatar" -> Avatar.routes
    )
    val files  = fileService[IO](FileService.Config("."))
    val routes = (server <+> files).orNotFound

    CORS.policy.withAllowOriginAll
      .withAllowCredentials(false)
      .apply(routes)
  }

  private def errorHandling(route: Kleisli[IO, Request[IO], Response[IO]]) =
    ErrorHandling.Recover.total(
      ErrorAction.log(
        route,
        messageFailureLogAction = (t, msg) =>
          IO.println(msg) >>
            IO.println(t),
        serviceErrorLogAction = (t, msg) =>
          IO.println(msg) >>
            IO.delay(t.printStackTrace())
      )
    )

//  val overrideConfiguration: ClientOverrideConfiguration =
//    ClientOverrideConfiguration.builder()
//      .apiCallTimeout(Duration.ofSeconds(30))
//      .apiCallAttemptTimeout(Duration.ofSeconds(20))
//      .retryPolicy(RetryPolicy.builder()
//        .numRetries(5)
//        .retryCondition(RetryCondition.defaultRetryCondition())
//        .build())
//      .build()
//
//  val httpClient: SdkAsyncHttpClient = NettyNioAsyncHttpClient.builder()
//    .connectionTimeout(Duration.ofSeconds(20))
//    .connectionAcquisitionTimeout(Duration.ofSeconds(20))
//    .connectionMaxIdleTime(Duration.ofSeconds(10))
//    .build()
//
//
//  def client(blobConfig: S3BlobConfig): S3AsyncClient = S3AsyncClient
//    .builder()
//    .region(Region.US_EAST_1)
//    .credentialsProvider(StaticCredentialsProvider.create(AwsBasicCredentials.create(blobConfig.accessKey, blobConfig.secretKey.value)))
//    .endpointOverride(URI.create(blobConfig.host))
//    .overrideConfiguration(overrideConfiguration)
//    .forcePathStyle(true)
//    .httpClient(httpClient)
//    .build()

//  def createBucket(s3: S3AsyncClient) =
//    IO.fromCompletableFuture(IO.delay(s3.createBucket(CreateBucketRequest.builder().bucket("mycelium").build())))
//      .void
//      .recoverWith {
//        case _: BucketAlreadyExistsException => IO.unit
//        case error => IO.delay(error.printStackTrace())
//      }

  val app: Resource[IO, Server] =
    for {
      cfg <- Resource.eval(AppConfig.config.load[IO])
      tx  <- Transactors.pg[IO](cfg.db)
//      s3Client = client(cfg.blob)
//      _ <- Resource.eval(createBucket(s3Client))
//      s3 <- Resource.eval(IO.fromOption(S3Store.builder[IO](s3Client).build.toOption)(new Throwable("Wat?")))
      repos = Repositories.fromTransactor(tx)
      server <- EmberServerBuilder
        .default[IO]
        .withHost(ipv4"0.0.0.0")
        .withPort(port"8080")
        .withHttpApp(errorHandling(httpApp(repos)))
        .build
    } yield server
}
