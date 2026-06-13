using System.Diagnostics;
using System.Dynamic;
using Microsoft.Extensions.Logging;
using Sequowl.Host.Models;
using Sequowl.Host.Repositories;
using Sequowl.Infrastructure.PostgreSql.Helpers;

namespace Sequowl.Infrastructure.PostgreSql.Repositories;

/// <summary>
/// Todo: maybe would be nice to return query errors even outside query.
/// I could show a snackbar to tell the user is something wrong with the connection string.
/// </summary>
/// <param name="builder"></param>
/// <param name="logger"></param>
public class PostgreSqlConnection(
    CommandBuilder builder,
    ILogger<PostgreSqlConnection> logger
) : IDatabaseConnection
{
    public async Task<IReadOnlyList<DatabaseInfo>> GetDatabases(CancellationToken cancellationToken)
    {
        var sql = @" SELECT d.datname AS name,
                            pg_get_userbyid(d.datdba) AS owner,
                            pg_encoding_to_char(d.encoding) AS encoding,
                            pg_database_size(d.datname) AS size_bytes
                   FROM pg_database d
                   ORDER BY d.datname;
                ";


        await using var command = builder.Build(sql);

        var result = new List<DatabaseInfo>();

        try
        {
            await using var reader = await command.ExecuteReaderAsync(cancellationToken);
            while (await reader.ReadAsync())
            {
                var database = new DatabaseInfo(name: reader.GetString(0),
                    owner: reader.IsDBNull(1) ? null : reader.GetString(1),
                    encoding: reader.IsDBNull(2) ? null : reader.GetString(2),
                    sizeBytes: reader.IsDBNull(3) ? null : reader.GetInt64(3)
                );

                result.Add(database);
            }
        }
        catch (Exception e)
        {
            logger.LogError(e.Message);
            throw;
        }

        return result.AsReadOnly();
    }

    public async Task<IReadOnlyList<TableInfo>> GetObjects(
        string databaseName,
        CancellationToken cancellationToken
    )
    {
        var sql = """
                  SELECT
                      n.nspname AS schema_name,
                      c.relname AS object_name,
                      CASE c.relkind
                          WHEN 'r' THEN 'TABLE'
                          WHEN 'v' THEN 'VIEW'
                          WHEN 'm' THEN 'MATERIALIZED_VIEW'
                          WHEN 'f' THEN 'FOREIGN_TABLE'
                          WHEN 'p' THEN 'PARTITIONED_TABLE'
                          WHEN 'S' THEN 'SEQUENCE'
                      END AS kind
                  FROM pg_class c
                  JOIN pg_namespace n ON n.oid = c.relnamespace
                  WHERE c.relkind IN ('r','v','m','f','p','S')
                  AND c.relname = @Database
                  ORDER BY n.nspname, c.relname;

                  """;

        await using var command = builder.Build(
            sql,
            param => param.AddWithValue("Database", databaseName)
        );

        var result = new List<TableInfo>();
        await using var reader = await command.ExecuteReaderAsync(cancellationToken);

        while (await reader.ReadAsync(cancellationToken))
        {
            var model = new TableInfo(
                name: reader.GetString(0),
                schema: reader.GetString(1),
                kind: reader.GetString(2)
            );
            result.Add(model);
        }

        return result.AsReadOnly();
    }

    public async Task<IReadOnlyList<ColumnInfo>> GetColumns(string databaseName, string tableName,
        CancellationToken cancellationToken)
    {
        var sql = @"
            SELECT
                column_name,
                data_type,
                udt_name,
                character_maximum_length,
                is_nullable,
                column_default,
                ordinal_position
            FROM information_schema.columns
            WHERE table_schema = 'public'
              AND table_name = 'vehicles'
            ORDER BY ordinal_position;
            ";

        await using var command = builder.Build(sql);
        var result = new List<ColumnInfo>();
        await using var reader = await command.ExecuteReaderAsync(cancellationToken);
        while (await reader.ReadAsync(cancellationToken))
        {
            var model = new ColumnInfo(
                reader.GetString(0),
                reader.GetString(1),
                reader.GetBoolean(2),
                reader.GetString(3),
                reader.GetInt32(4),
                reader.GetBoolean(5)
            );
            result.Add(model);
        }

        return result.AsReadOnly();
    }


    private static readonly string[] WriteCommands =
    [
        "INSERT",
        "UPDATE",
        "DELETE",
        "DROP",
        "ALTER",
        "CREATE",
        "TRUNCATE",
        "GRANT",
        "REVOKE"
    ];

    private static bool IsReadOnly(string sql)
    {
        var trimmed = sql.TrimStart().ToUpperInvariant();
        return !WriteCommands.Any(trimmed.StartsWith);
    }

    /// <summary>
    /// Very ugly method. I would like to refactor this later.
    /// </summary>
    /// <param name="sqlCommand"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    public async Task<SqlQueryResult> ExecuteQuery(string sqlCommand, CancellationToken cancellationToken)
    {
        await using var command = builder.Build(sqlCommand);

        var sw = Stopwatch.StartNew();

        try
        {
            if (!IsReadOnly(sqlCommand))
            {
                var affected = await command.ExecuteNonQueryAsync(cancellationToken);
                sw.Stop();
                return SqlQueryResult.Altered(affected, sw.ElapsedMilliseconds);
            }

            await using var reader = await command.ExecuteReaderAsync(cancellationToken);
            var result = new List<dynamic>();

            while (await reader.ReadAsync(cancellationToken))
            {
                IDictionary<string, object?> row = new ExpandoObject();

                for (var i = 0; i < reader.FieldCount; i++)
                    row[reader.GetName(i)] = reader.IsDBNull(i) ? null : reader.GetValue(i);

                result.Add(row);
            }

            return SqlQueryResult.Query(data: result.AsReadOnly(), timeElapsed: sw.ElapsedMilliseconds);
        }
        catch (Exception e)
        {
            sw.Stop();
            return SqlQueryResult.Failure(error: e.Message, sw.ElapsedMilliseconds);
        }
    }
}