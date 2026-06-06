namespace Sequowl.Host.Models;

public sealed record TableInfo
{
    public TableInfo(string schema, string name, string kind)
    {
        Schema = schema;
        Name = name;
        Kind = kind switch
        {
            "VIEW" => TableKind.View,
            "TABLE" => TableKind.Table,
            "MATERIALIZED_VIEW" => TableKind.MaterializedView,
            _ => TableKind.Table
        };
    }

    public string Schema { get; init; }
    public string Name { get; set; }

    public TableKind Kind { get; init; }
}