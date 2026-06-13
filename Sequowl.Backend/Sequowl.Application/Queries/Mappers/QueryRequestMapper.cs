namespace Sequowl.Application.Queries.Mappers;


public interface IMapper<in TRequest, out TCommand>
    where TCommand : class
    where TRequest : class
{
    TCommand Map(TRequest query);
}