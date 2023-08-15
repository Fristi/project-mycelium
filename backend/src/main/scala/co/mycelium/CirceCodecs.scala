package co.mycelium

import co.mycelium.domain.{
  Station,
  StationDetails,
  StationEvent,
  StationInsert,
  StationLog,
  StationMeasurement,
  StationUpdate,
  Watering,
  WateringSchedule
}
import cron4s.{Cron, CronExpr}
import io.circe.generic.extras.Configuration
import io.circe.generic.extras.semiauto.deriveConfiguredCodec
import io.circe.{Codec, Decoder, Encoder}
import io.circe.generic.semiauto.deriveCodec

import scala.concurrent.duration.{Duration, FiniteDuration}

object CirceCodecs {
  implicit val genDevConfig: Configuration =
    Configuration.default.withDiscriminator("_type")

  implicit val codecFiniteDuration: Codec[FiniteDuration] = {
    def decode(in: String) =
      Option(Duration(in)).collect { case s: FiniteDuration => s }

    Codec
      .from(Decoder.decodeString, Encoder.encodeString)
      .iemap(s => decode(s).toRight("Invalid duration"))(_.toString())
  }

  implicit val codecCronExpr: Codec[CronExpr] =
    Codec
      .from(Decoder.decodeString, Encoder.encodeString)
      .iemap(s => Cron.parse(s).left.map(_.getMessage))(_.toString)

  implicit val codecWateringSchedule: Codec[WateringSchedule] = deriveConfiguredCodec
  implicit val codecStationEvent: Codec[StationEvent]         = deriveConfiguredCodec
  implicit val codecStationLog: Codec[StationLog]             = deriveCodec
  implicit val codecStationReading: Codec[StationMeasurement] = deriveCodec

  implicit val codecInsert: Codec[StationInsert]          = deriveCodec
  implicit val codecUpdate: Codec[StationUpdate]          = deriveCodec
  implicit val codecWatering: Codec[Watering]             = deriveCodec
  implicit val codecStation: Codec[Station]               = deriveCodec
  implicit val codecStationDetails: Codec[StationDetails] = deriveCodec
}
