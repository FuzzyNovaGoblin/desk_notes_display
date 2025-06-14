#include <Arduino.h>
#include <Fonts/FreeMonoBold9pt7b.h>
#include <GxEPD2_3C.h>
#include <GxEPD2_4C.h>
#include <GxEPD2_7C.h>
#include <GxEPD2_BW.h>
#include <HTTPClient.h>
#include <WiFiMulti.h>

#include "GxEPD2_display_selection_new_style.h"
#include "config.h"

WiFiMulti wifiMulti;
String content;
const String serverUri = "http://" SERVER_IP ":" SERVER_PORT;

void setup() {

  content = "";
  Serial.begin(115200);

  Serial.println();
  Serial.println();
  Serial.println();

  wifiMulti.addAP(WIFI_SSID, WIFI_PW);
  display.init(115200, true, 2, false);
}

void displayText(String dispText) {
  Serial.printf("display:\n%s\n", dispText);
  content = dispText;
  display.setRotation(0);
  display.setFont(&FreeMonoBold9pt7b);
  display.setTextColor(GxEPD_BLACK);
  int16_t tbx, tby;
  uint16_t tbw, tbh;
  display.getTextBounds(dispText, 0, 0, &tbx, &tby, &tbw, &tbh);
  uint16_t x, y = 0;
  display.setFullWindow();
  display.firstPage();
  do {
    display.fillScreen(GxEPD_WHITE);
    display.setCursor(x, y);
    display.print(dispText);
  } while (display.nextPage());
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
        Serial.println(payload);
        if (payload != content) {
          displayText(payload);
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