namespace Sequowl.Host.Connections;

public interface IUnityOfWork : IDisposable
{
    public Task BeginTransaction(CancellationToken ct = default);
    public Task Commit(CancellationToken ct = default);
    public Task Rollback(CancellationToken ct = default);
}