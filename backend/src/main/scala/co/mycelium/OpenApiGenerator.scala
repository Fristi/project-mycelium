package co.mycelium

import co.mycelium.endpoints.Stations
import sttp.apispec.openapi.OpenAPI
import sttp.tapir.docs.openapi.OpenAPIDocsInterpreter
import sttp.apispec.openapi.circe._
import sttp.apispec.openapi.Server
import io.circe.syntax._

import java.nio.file.{Files, OpenOption, Path}

object OpenApiGenerator extends App {
  val endpoints = Stations.endpoints.all.map(_.endpoint)
  val docs: OpenAPI = OpenAPIDocsInterpreter()
    .toOpenAPI(endpoints, "Mycelium API", "1.0.0")
//    .addServer(Server("https://mycelium.app.dev", Some("Production server")))

  val path = Path.of("openapi.json")

  Files.write(path, docs.asJson.noSpaces.getBytes())
}
