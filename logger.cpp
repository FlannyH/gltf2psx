#include <cstdint>
#include <Windows.h>

void set_console_colour(const uint32_t colour)
{
	HANDLE hConsole = GetStdHandle(STD_OUTPUT_HANDLE);
	SetConsoleTextAttribute(hConsole, *(WORD*)(&colour));
}
