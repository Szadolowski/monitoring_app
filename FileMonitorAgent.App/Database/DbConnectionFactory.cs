using System;
using System.Data;
using System.Threading.Tasks;
using DotNetEnv;
using Npgsql;

namespace FileMonitorAgent.App.Database
{
    public class DbConnectionFactory
    {
        private readonly string _connectionString;

        public DbConnectionFactory()
        {
            Env.Load();

            // Budujemy Connection String z pojedynczych zmiennych
            var host = Environment.GetEnvironmentVariable("DB_HOST");
            var port = Environment.GetEnvironmentVariable("DB_PORT") ?? "5432";
            var user = Environment.GetEnvironmentVariable("DB_USER");
            var pass = Environment.GetEnvironmentVariable("DB_PASS");
            var name = Environment.GetEnvironmentVariable("DB_NAME");

            if (string.IsNullOrEmpty(host) || string.IsNullOrEmpty(user))
            {
                throw new InvalidOperationException(
                    "Błąd konfiguracji: Brak zmiennych DB_ w pliku .env"
                );
            }

            _connectionString =
                $"Host={host};Port={port};Username={user};Password={pass};Database={name};";
        }

        public async Task<IDbConnection> CreateConnectionAsync()
        {
            var connection = new NpgsqlConnection(_connectionString);
            await connection.OpenAsync();
            return connection;
        }
    }
}
