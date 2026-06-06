namespace Sequowl.Host.Repositories;

public record Optional<T> where T : class
{
    private readonly T? _value;
    public bool IsPresent => _value != null;
    public T? Value => _value;

    public Optional(T value)
    {
        _value = value;
    }

    public static Optional<T> For(T value)
    {
        return new Optional<T>(value);
    }
}