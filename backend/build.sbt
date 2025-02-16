scalaVersion := "2.13.12"
name         := "backend"
organization := "co.mycelium"
version      := "1.0"

val doobieVersion = "1.0.0-RC5"

libraryDependencies ++= Seq(
  "org.tpolecat"                  %% "doobie-core"           % doobieVersion,
  "org.tpolecat"                  %% "doobie-postgres"       % doobieVersion,
  "org.tpolecat"                  %% "doobie-hikari"         % doobieVersion,
  "org.tpolecat"                  %% "doobie-postgres-circe" % doobieVersion,
  "org.typelevel"                 %% "cats-tagless-macros"   % "0.15.0",
  "com.github.alonsodomin.cron4s" %% "cron4s-core"           % "0.8.2",
  "org.http4s"                    %% "http4s-dsl"            % "0.23.25",
  "org.http4s"                    %% "http4s-ember-server"   % "0.23.25",
  "com.softwaremill.sttp.tapir"   %% "tapir-http4s-server"   % "1.9.11",
  "com.softwaremill.sttp.tapir"   %% "tapir-json-circe"      % "1.9.11",
  "org.flywaydb"                   % "flyway-core"           % "10.8.1",
  "io.circe"                      %% "circe-generic-extras"  % "0.14.3",
  "ch.qos.logback"                 % "logback-classic"       % "1.5.16",
  "com.github.jwt-scala"          %% "jwt-core"              % "10.0.4",
  "com.github.jwt-scala"          %% "jwt-circe"             % "10.0.4",
  "com.auth0"                      % "jwks-rsa"              % "0.22.1",
  "com.github.fs2-blobstore"      %% "s3"                    % "0.9.15",
  "is.cir"                        %% "ciris"                 % "3.5.0",
  "com.github.cb372"              %% "cats-retry"            % "3.1.3",
  "io.sentry"                      % "sentry-logback"        % "7.4.0",
  "org.postgresql"                 % "postgresql"            % "42.7.2",
  "com.softwaremill.sttp.tapir"   %% "tapir-openapi-docs"    % "1.9.10",
  "com.softwaremill.sttp.apispec" %% "openapi-circe-yaml"    % "0.11.7"
)

Compile / scalacOptions ++= {
  CrossVersion.partialVersion(scalaVersion.value) match {
    case Some((2, n)) if n >= 13 => "-Ymacro-annotations" :: Nil
    case _                       => Nil
  }
}

assembly / assemblyMergeStrategy := {
  case PathList("META-INF", entry) if entry.startsWith("services") => MergeStrategy.concat
  case PathList("META-INF", _)                                     => MergeStrategy.discard
  case _                                                           => MergeStrategy.first
}
