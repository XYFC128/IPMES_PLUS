package org.ipmes.pattern;

import org.ipmes.pattern.Preprocess;
import org.json.JSONObject;
import org.junit.Test;

import static org.junit.Assert.*;

public class SignatureExtractionTest {
    @Test
    public void nodeSignature() {
        assertEquals(
                "Process::hello.sh",
                Preprocess.extractNodeSignature(new JSONObject("{\"properties\":{\"type\":\"Process\",\"subtype\":\"process\",\"name\":\"hello.sh\"}}"))
        );
        assertEquals(
                "Artifact::file::/etc/password",
                Preprocess.extractNodeSignature(new JSONObject("{\"properties\":{\"type\":\"Artifact\",\"subtype\":\"file\",\"path\":\"/etc/password\"}}"))
        );
        assertEquals(
                "Artifact::network socket::10.2.2.1:8787",
                Preprocess.extractNodeSignature(new JSONObject("{\"properties\":{\"type\":\"Artifact\",\"subtype\":\"network socket\",\"remote address\":\"10.2.2.1\",\"remote port\":\"8787\"}}"))
        );
        assertEquals(
                "Unknown::",
                Preprocess.extractNodeSignature(new JSONObject("{\"properties\":{\"type\":\"Unknown\"}}"))
        );
    }
}
