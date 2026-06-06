using System.Data;
using Sequowl.Host.Connections;

namespace Sequowl.Infrastructure.PostgreSql.Connection;

public class UnityOfWork(PostgreSqlSession session) : IUnityOfWork
{
    public async Task BeginTransaction(CancellationToken ct = default)
    {
        if (session.Connection.State != ConnectionState.Open)
            await session.Connection.OpenAsync(ct);

        var tx = await session.Connection.BeginTransactionAsync(ct);
    }

    public async Task Commit(CancellationToken ct = default)
    {
        if (session.Transaction is null)
            throw new InvalidOperationException("No active transaction.");

        await session.Transaction.CommitAsync(ct);
        await session.Transaction.DisposeAsync();
        // session.ClearTransaction();
    }

    public async Task Rollback(CancellationToken ct = default)
    {
        if (session.Transaction is null) return;

        await session.Transaction.RollbackAsync(ct);
        await session.Transaction.DisposeAsync();
        // session.ClearTransaction();
    }

    public void Dispose()
    {
        // session.ClearTransaction();
    }
}