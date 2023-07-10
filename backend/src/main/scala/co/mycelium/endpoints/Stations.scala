package co.mycelium.endpoints

import cats.effect.IO
import co.mycelium.CirceCodecs._
import co.mycelium.db.Repositories
import co.mycelium.domain._
import cron4s.CronExpr
import org.http4s.HttpRoutes
import sttp.tapir._
import sttp.tapir.generic.Configuration
import sttp.tapir.generic.auto._
import sttp.tapir.json.circe._
import sttp.tapir.server.http4s.Http4sServerInterpreter

import java.time.Instant
import java.util.UUID
import scala.concurrent.duration.FiniteDuration

object Stations extends TapirSchemas {

  object endpoints {
    val stations = base.in("stations")

    val list = stations.get.out(jsonBody[List[Station]])
    val add = stations.post.in(jsonBody[StationInsert])
    val update = stations.put.in(path[UUID]("stationId")).in(jsonBody[StationUpdate])
    val delete = stations.put.in(path[UUID]("stationId"))
    val checkIn = stations.in(path[UUID]("stationId")).in("checkin").put.in(jsonBody[LogReadings]).out(jsonBody[Watering])
    val watered = stations.in(path[UUID]("stationId")).in("watered").post.in(jsonBody[Watering])
    val log = stations.in(path[UUID]("stationId")).in("log").in(query[Option[Long]]("page")).out(jsonBody[List[StationLog]])

    val all = Set(list, add, update, delete, checkIn, watered)
  }

  def routes(repositories: Repositories[IO]): HttpRoutes[IO] = {

    val userId = "1"

    val list = endpoints.list.serverLogic[IO](_ => repositories.stations.listByUserId(userId).map(Right(_)))
    val add = endpoints.add.serverLogic[IO] { insert =>
      val id = UUID.randomUUID()
      val created = Instant.now()

      repositories.stations.insert(insert.toStation(id, created, userId), created).as(Right(()))
    }

    val delete = endpoints.delete.serverLogic[IO](id => repositories.stations.delete(id).as(Right(())))

    val checkin = endpoints.checkIn.serverLogic[IO] { case (id, readings) =>
      for {
        stationOpt <- repositories.stations.findById(id)
        watering <- stationOpt match {
          case Some(station) =>
            station.wateringSchedule match {
              case WateringSchedule.Interval(_, _) => IO(Watering(None))
              case WateringSchedule.Threshold(belowSoilPf, period) =>
                if(readings.readings.lastOption.exists(_.soilPf < belowSoilPf)) IO(Watering(Some(period))) else IO(Watering(None))
            }
          case None => IO(Watering(None))
        }
      } yield Right(watering)
    }

    val watered = endpoints.watered.serverLogic[IO] { case (id, request) =>
      request.watering match {
        case Some(watered) => repositories.stationLog.insert(StationLog(id, Instant.now(), StationEvent.Watered(watered))).as(Right(()))
        case None => IO.unit.as(Right(()))
      }
    }

    val log = endpoints.log.serverLogic[IO] { case (id, page) => repositories.stationLog.listByStation(id, page.getOrElse(0L) * 30).map(Right(_)) }

    Http4sServerInterpreter[IO]().toRoutes(List(list, add, delete, log, watered, checkin))

  }
}

trait TapirSchemas {
  implicit val customConfiguration: Configuration =
    Configuration.default.withDiscriminator("_type")

  implicit val schemaCronExpr: Schema[CronExpr] = Schema.string
  implicit val schemaFiniteDuration: Schema[FiniteDuration] = Schema.string
  implicit val schemaWateringSchedule: Schema[WateringSchedule] = Schema.derived
}
