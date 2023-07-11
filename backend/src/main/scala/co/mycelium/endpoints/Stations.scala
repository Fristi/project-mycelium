package co.mycelium.endpoints

import cats.effect.IO
import co.mycelium.CirceCodecs._
import co.mycelium.db.Repositories
import co.mycelium.domain._
import cron4s.CronExpr
import cron4s.lib.javatime.javaTemporalInstance
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
    val get = stations.get.in(path[UUID]("stationId")).in(query[Option[MeasurementPeriod]]("period")).out(jsonBody[StationDetails])
    val update = stations.put.in(path[UUID]("stationId")).in(jsonBody[StationUpdate])
    val delete = stations.put.in(path[UUID]("stationId"))
    val checkIn = stations.in(path[UUID]("stationId")).in("checkin").put.in(jsonBody[List[StationMeasurement]]).out(jsonBody[Watering])
    val watered = stations.in(path[UUID]("stationId")).in("watered").post.in(jsonBody[Watering])
    val log = stations.in(path[UUID]("stationId")).in("log").in(query[Option[Long]]("page")).out(jsonBody[List[StationLog]])

    val all = Set(list, add, update, delete, checkIn, watered)
  }

  def routes(repos: Repositories[IO]): HttpRoutes[IO] = {


    val list = endpoints.list.serverLogic(at => _ => repos.stations.listByUserId(at.sub).map(Right(_)))
    val add = endpoints.add.serverLogic { at => insert =>
      val id = UUID.randomUUID()
      val created = Instant.now()

      repos.stations.insert(insert.toStation(id, created, at.sub), created).as(Right(()))
    }

    val delete = endpoints.delete.serverLogic(at => id => repos.stations.delete(id, at.sub).as(Right(())))

    val checkin = endpoints.checkIn.serverLogic { at => {
      case (id, measurements) =>
          for {
            stationOpt <- repos.stations.findById(id, at.sub)
            _ <- repos.measurements.insertMany(id, measurements)
            watering <- stationOpt match {
              case Some(station) =>
                station.wateringSchedule match {
                  case WateringSchedule.Interval(schedule, period) =>
                    repos.stationLog.lastTimeWatered(id).flatMap {
                      case Some(lastTime) => schedule.next(lastTime) match {
                        case Some(nextTime) if nextTime.isAfter(Instant.now()) => IO(Watering(Some(period)))
                        case _ => IO(Watering(None))
                      }
                      case None => IO(Watering(None))
                    }

                  case WateringSchedule.Threshold(belowSoilPf, period) =>
                    if(measurements.lastOption.exists(_.soilPf < belowSoilPf)) IO(Watering(Some(period))) else IO(Watering(None))
                }
              case None => IO(Watering(None))
            }
          } yield Right(watering)
      }
    }

    val watered = endpoints.watered.serverLogic { at => {
      case (id, request) =>
          request.watering match {
            case Some(watered) => repos.stationLog.insert(StationLog(id, Instant.now(), StationEvent.Watered(watered))).as(Right(()))
            case None => IO.unit.as(Right(()))
          }
      }
    }

    val log = endpoints.log.serverLogic { at => {
      case (id, page) =>
        repos.stationLog.listByStation(id, page.getOrElse(0L) * 30).map(Right(_))
      }
    }

    val details = endpoints.get.serverLogic { at => {
      case (id, period) =>
        repos.stations.findById(id, at.sub).flatMap {
          case Some(station) =>
            repos.measurements.avg(id, period.getOrElse(MeasurementPeriod.LastTwentyFourHours)).map(measurements => Right(StationDetails(station, measurements)))
          case None =>
            IO.delay(Left(()))
        }
    }}

    Http4sServerInterpreter[IO]().toRoutes(List(list, add, delete, log, watered, checkin, details))
  }
}

trait TapirSchemas {
  implicit val customConfiguration: Configuration =
    Configuration.default.withDiscriminator("_type")

  implicit val schemaCronExpr: Schema[CronExpr] = Schema.string
  implicit val schemaFiniteDuration: Schema[FiniteDuration] = Schema.string
  implicit val schemaWateringSchedule: Schema[WateringSchedule] = Schema.derived

  implicit val codecMeasurementPeriod: Codec[String, MeasurementPeriod, CodecFormat.TextPlain] =
    Codec.string.map(Mapping.fromDecode((str: String) => DecodeResult.fromOption(MeasurementPeriod.fromString(str)))(_.repr))
}
