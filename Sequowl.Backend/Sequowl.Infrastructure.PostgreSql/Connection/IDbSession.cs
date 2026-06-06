using Npgsql;

namespace Sequowl.Infrastructure.PostgreSql.Connection;

//TODO: move it to the core and go after abstractions.
// public interface IDbSession : IDisposable, IAsyncDisposable
// {
//     NpgsqlConnection Connection { get; }
//     NpgsqlTransaction? Transaction { get; }
// }

/// <summary>
/// This will be possible when using DI factories.
/// </summary>
/// <param name="connectionString"></param>
public sealed class PostgreSqlSession
{
    public Task OpenAsync(CancellationToken ct) => Connection.OpenAsync(ct);
    public NpgsqlConnection Connection { get; }
    public NpgsqlTransaction? Transaction { get; }

    public void Dispose() => Connection.Dispose();
    public ValueTask DisposeAsync() => Connection.DisposeAsync();
}