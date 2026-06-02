var builder = WebApplication.CreateBuilder(args);

builder.Services.AddGrpc();

var app = builder.Build();

app.Run();

// gRPC stands for gRPC Remote Procedure Call
// gRPC works with Protocol Buffer/ Protobuf as serialization.
// proto is a contract for gRPC
