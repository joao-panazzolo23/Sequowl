namespace Sequowl.Host.Models;

public sealed record ColumnInfo
{
    public ColumnInfo(
        string name,
        string dbTypeName,
        bool isNullable,
        string defaultValue,
        int ordinalPosition,
        bool isPrimaryKey
    )
    {
        Name = name;
        DbTypeName = dbTypeName;
        IsNullable = isNullable;
        DefaultValue = defaultValue;
        OrdinalPosition = ordinalPosition;
        IsPrimaryKey = isPrimaryKey;
    }

    public string Name { get; init; }
    public string DbTypeName { get; init; }
    public bool IsNullable { get; init; }
    public bool IsPrimaryKey { get; init; }
    public string? DefaultValue { get; init; }
    public int OrdinalPosition { get; init; }
}