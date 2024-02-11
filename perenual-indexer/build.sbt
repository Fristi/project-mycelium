scalaVersion := "2.13.12"
name         := "perenual-indexer"
organization := "co.mycelium"
version      := "1.0"
val doobieVersion = "1.0.0-RC5"

lazy val root = (project in file("."))
  .settings(
    name := "perenual-indexer",
    libraryDependencies ++= Seq(
      "dev.zio"                       %% "zio-streams"      % "2.0.16",
      "dev.zio"                       %% "zio-interop-cats" % "23.0.0.5",
      "org.tpolecat"                  %% "doobie-core"      % doobieVersion,
      "org.tpolecat"                  %% "doobie-postgres"  % doobieVersion,
      "org.tpolecat"                  %% "doobie-hikari"    % doobieVersion,
      "ch.qos.logback"                 % "logback-classic"  % "1.4.14",
      "com.github.cb372"              %% "cats-retry"       % "3.1.0",
      "com.softwaremill.sttp.client3" %% "zio"              % "3.9.0",
      "com.softwaremill.sttp.client3" %% "zio-json"         % "3.9.0"
    )
  )

assembly / assemblyMergeStrategy := {
  case PathList("META-INF", entry) if entry.startsWith("services") => MergeStrategy.concat
  case PathList("META-INF", _)                                     => MergeStrategy.discard
  case _                                                           => MergeStrategy.first
}
