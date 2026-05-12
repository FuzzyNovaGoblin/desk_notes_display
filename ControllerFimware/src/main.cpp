#include <Arduino.h>
#include <GxEPD2_3C.h>
#include <GxEPD2_4C.h>
#include <GxEPD2_7C.h>
#include <GxEPD2_BW.h>
#include <HTTPClient.h>
#include <WiFiMulti.h>
#include <iostream>
#include <sstream>
#include <string>

#include "GxEPD2_display_selection_new_style.h"
#include "JetBrains_Mono_Bold_Nerd_Font_Complete_Mono15pt7b.h"
#include "config.h"

void displayText(String);

WiFiMulti wifiMulti;
String last_displayed;
const String serverUri = "http://" SERVER_IP ":" SERVER_PORT;

const GFXfont &FONT = JetBrains_Mono_Bold_Nerd_Font_Complete_Mono15pt7b;
const int DISP_W = 800;
const int DISP_H = 480;
const int MAX_STRING_WIDTH = ceil(((float)DISP_W) / FONT.glyph[0].xAdvance);

void setup() {

  last_displayed = "";
  Serial.begin(115200);

  Serial.println();
  Serial.println();
  Serial.println();

  wifiMulti.addAP(WIFI_SSID, WIFI_PW);
  display.init(115200, true, 2, false);
  delay(500);
}

void loop() {
  // wait for WiFi connection
  if ((wifiMulti.run() == WL_CONNECTED)) {

    HTTPClient http;

    Serial.print("[HTTP] begin...\n");
    http.begin(serverUri);
    int status = http.GET();
    if (status > 0) {
      Serial.printf("[HTTP] GET... code: %d\n", status);

      if (status == HTTP_CODE_OK) {
        String payload = http.getString();
        if (payload != last_displayed) {
          last_displayed = payload;

          String rebuild_string = "";

          int current_width = 0;
          for (int i = 0; i < payload.length(); i++) {
            current_width += 1;
            // Serial.printf("cur:%i\n", current_width);
            switch (payload[i]) {
            case '\n':
              rebuild_string += '\n';
              current_width = 0;
              break;

            default:
              if (current_width <= MAX_STRING_WIDTH) {
                rebuild_string += payload[i];
                // Serial.printf("added char\n");
              }else{
                Serial.printf("drop char\n");
              }
              break;
            }
            Serial.flush();
          }

          Serial.println(rebuild_string);
          displayText(rebuild_string);
        }
      }
    } else {
      Serial.printf("[HTTP] GET... failed, error: %s\n",
                    http.errorToString(status).c_str());
    }

    http.end();

    delay(10000);

  } else {

    delay(2000);
  }
};

void displayText(String dispText) {
  Serial.printf("MAX_STRING_WIDTH: %i\n", MAX_STRING_WIDTH);

  Serial.printf("display:\n%s\n", dispText);
  display.setRotation(2);
  display.setFont(&FONT);
  display.setTextColor(GxEPD_BLACK);
  int16_t tbx, tby;
  uint16_t tbw, tbh;
  display.getTextBounds(dispText, 0, 0, &tbx, &tby, &tbw, &tbh);
  uint16_t x = 0;
  uint16_t y = FONT.yAdvance * 0.75;
  display.setFullWindow();
  display.firstPage();
  do {
    display.fillScreen(GxEPD_WHITE);
    display.setCursor(x, y);
    display.print(dispText);
  } while (display.nextPage());
}
