scalaVersion := "2.13.10"
name := "backend"
organization := "co.mycelium"
version := "1.0"

val doobieVersion = "1.0.0-RC4"

libraryDependencies ++= Seq(
    "org.tpolecat" %% "doobie-core" % doobieVersion,
    "org.tpolecat" %% "doobie-postgres" % doobieVersion,
    "org.tpolecat" %% "doobie-hikari" % doobieVersion,
    "org.http4s" %% "http4s-ember-server" % "0.23.18",
    "com.softwaremill.sttp.tapir" %% "tapir-http4s-server" % "1.6.0"
)