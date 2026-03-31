package com.claudiator.app

import com.claudiator.app.util.URLValidator
import org.junit.Assert.*
import org.junit.Test

class URLValidatorTest {

    @Test
    fun `empty input returns null`() {
        assertNull(URLValidator.cleanAndValidate(""))
    }

    @Test
    fun `whitespace-only input returns null`() {
        assertNull(URLValidator.cleanAndValidate("   "))
        assertNull(URLValidator.cleanAndValidate("\t\n"))
    }

    @Test
    fun `trailing slash is removed`() {
        assertEquals("https://example.com", URLValidator.cleanAndValidate("https://example.com/"))
    }

    @Test
    fun `no trailing slash is left unchanged`() {
        assertEquals("https://example.com", URLValidator.cleanAndValidate("https://example.com"))
    }

    @Test
    fun `localhost gets http prefix`() {
        assertEquals("http://localhost:3000", URLValidator.cleanAndValidate("localhost:3000"))
    }

    @Test
    fun `local domain gets http prefix`() {
        assertEquals("http://server.local", URLValidator.cleanAndValidate("server.local"))
        assertEquals("http://test.local:3000", URLValidator.cleanAndValidate("test.local:3000"))
    }

    @Test
    fun `loopback gets http prefix`() {
        assertEquals("http://127.0.0.1:8080", URLValidator.cleanAndValidate("127.0.0.1:8080"))
    }

    @Test
    fun `remote hostname gets https prefix`() {
        assertEquals("https://example.com", URLValidator.cleanAndValidate("example.com"))
        assertEquals("https://api.myapp.io", URLValidator.cleanAndValidate("api.myapp.io"))
    }

    @Test
    fun `existing http prefix is preserved`() {
        assertEquals("http://example.com", URLValidator.cleanAndValidate("http://example.com"))
    }

    @Test
    fun `existing https prefix is preserved`() {
        assertEquals("https://example.com", URLValidator.cleanAndValidate("https://example.com"))
    }

    @Test
    fun `whitespace trimming and trailing slash combined`() {
        assertEquals("https://example.com", URLValidator.cleanAndValidate("  https://example.com/  "))
    }

    @Test
    fun `localhost with trailing slash gets cleaned`() {
        assertEquals("http://localhost:3000", URLValidator.cleanAndValidate("localhost:3000/"))
    }

    @Test
    fun `invalid URL returns null`() {
        assertNull(URLValidator.cleanAndValidate("http://exam ple.com"))
    }
}
