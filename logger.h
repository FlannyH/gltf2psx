#pragma once
//#include <imgui.h>
#include <iostream>
#include <string>
#include <vector>
#include <glm/vec2.hpp>
#include <glm/vec3.hpp>
#include "resource_handler_structs.h"

void set_console_colour(const uint32_t colour);

class Logger
{
public:
	struct Message
	{
		Pixel32 colour;
		std::string text;
	};
	inline static std::vector<Message> messages;

	static void logf(const char* fmt...)
	{

		va_list args;
		va_start(args, fmt);

		std::string msg = "";

		while (*fmt != '\0') {

			if (*fmt == '%')
			{
				++fmt;
				if (*fmt == 'i' || *fmt == 'I') {
					int i = va_arg(args, int);
					msg += std::to_string(i);
				}
				else if (*fmt == 'f' || *fmt == 'F') {
					double f = va_arg(args, double);
					msg += std::to_string(f);
				}
				else if (*fmt == 'c' || *fmt == 'C') {
					const char c = va_arg(args, char);
					msg += c;
				}
				else if (*fmt == 's' || *fmt == 'S') {
					std::string s = va_arg(args, char*);
					msg += s;
				}

				if (*fmt == 'v' || *fmt == 'V')
				{
					++fmt;
					if (*fmt == '2') {
						glm::vec2 v2 = va_arg(args, glm::vec2);
						msg += std::to_string(v2.x) + ", " + std::to_string(v2.y);
					}
					else if (*fmt == '3') {
						glm::vec3 v3 = va_arg(args, glm::vec3);
						msg += std::to_string(v3.x) + ", " + std::to_string(v3.y) + ", " + std::to_string(v3.z);
					}
				}
			}
			else
			{
				msg += *fmt;
			}
			++fmt;
		}

		va_end(args);
		Message message{};
		if (msg._Starts_with("[ERROR]"))
		{
			set_console_colour(0x0C);
			message.colour = { 255, 0, 0, 255 };
		}
		message.text = msg;

		messages.push_back(message);

		std::cout << msg << std::endl;
		set_console_colour(0x0F);
	}
};

