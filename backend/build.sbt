scalaVersion := "2.13.10"
name := "backend"
organization := "co.mycelium"
version := "1.0"

val doobieVersion = "1.0.0-RC4"

libraryDependencies ++= Seq(
    "org.tpolecat" %% "doobie-core" % doobieVersion,
    "org.tpolecat" %% "doobie-postgres" % doobieVersion,
    "org.tpolecat" %% "doobie-hikari" % doobieVersion,
    "org.tpolecat" %% "doobie-postgres-circe" % doobieVersion,
    "org.typelevel" %% "cats-tagless-macros" % "0.15.0",
    "com.github.alonsodomin.cron4s" %% "cron4s-core" % "0.6.1",
    "org.http4s" %% "http4s-ember-server" % "0.23.18",
    "com.softwaremill.sttp.tapir" %% "tapir-http4s-server" % "1.6.0",
    "com.softwaremill.sttp.tapir" %% "tapir-json-circe" % "1.6.0",
    "org.flywaydb" % "flyway-core" % "9.16.0",
    "io.circe" %% "circe-generic-extras" % "0.14.3"
)

Compile / scalacOptions ++= {
    CrossVersion.partialVersion(scalaVersion.value) match {
        case Some((2, n)) if n >= 13 => "-Ymacro-annotations" :: Nil
        case _ => Nil
    }
}