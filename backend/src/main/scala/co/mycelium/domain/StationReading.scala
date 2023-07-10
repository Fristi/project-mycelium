package co.mycelium.domain

import java.time.Instant

final case class StationReading(on: Instant, soilPf: Double, lux: Double, humidity: Double, temperature: Double)