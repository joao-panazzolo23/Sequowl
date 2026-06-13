namespace Sequowl.Host.Models;

public sealed record DatabaseInfo
{
    public DatabaseInfo(
        string name,
        string? owner,
        string? encoding,
        long? sizeBytes
    )
    {
        Name = name;
        Owner = owner;
        Encoding = encoding;
        SizeBytes = sizeBytes;
    }

    public string Name { get; init; }
    public string? Owner { get; init; }
    public string? Encoding { get; init; }
    public long? SizeBytes { get; init; }
}