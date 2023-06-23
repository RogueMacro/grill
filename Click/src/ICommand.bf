using System;

namespace Click;

interface ICommand
{
	Result<void> Run();
}