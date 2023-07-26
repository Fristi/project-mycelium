package co.mycelium.endpoints

import blobstore.s3.S3Store
import blobstore.url.{Authority, Url}
import cats.effect.IO
import org.http4s.{HttpRoutes, Response}
import org.http4s.dsl.io._
import fs2._

import java.util.UUID

object Avatar {

//  private def mkUrl(uuid: UUID) =
//    Url("s3", Authority.unsafe("mycelium"), blobstore.url.Path(s"$uuid"))

//  def routes(s3: S3Store[IO]) = HttpRoutes.of[IO] {
  def routes = HttpRoutes.of[IO] {
    case GET -> Root / UUIDVar(uuid) =>

      val stream = getClass.getResourceAsStream("/placeholder.png")
      val placeholder = io.readInputStream[IO](IO.delay(stream), 1024)
//      val avatar = s3.get(mkUrl(uuid), 1024)
//      val payload = avatar.handleErrorWith(_ => placeholder)
      val payload = placeholder

      IO.delay(Response(body = payload))

//    case req @ PUT -> Root / UUIDVar(uuid) =>
//      req.body.through(s3.put(mkUrl(uuid), true)).compile.drain.as(Response())
  }
}
