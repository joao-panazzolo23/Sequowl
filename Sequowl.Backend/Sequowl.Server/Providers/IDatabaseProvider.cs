using Sequowl.Host.Models;

namespace Sequowl.Host.Providers;

public interface IDatabaseProvider
{
    Task<IDatabaseProvider> CreateSessionAsync(
        ConnectionDefinition definition,
        CancellationToken ct
    );
}