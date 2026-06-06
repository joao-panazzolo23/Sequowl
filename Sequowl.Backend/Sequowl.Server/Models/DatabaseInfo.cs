namespace Sequowl.Host.Models;

public sealed record DatabaseInfo
{
    public required string Name { get; init; }
    public string? Owner { get; init; }
    public string? Encoding { get; init; }
    public long? SizeBytes { get; init; }
}