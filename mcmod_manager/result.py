"""A rust-like result type."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable


class ResultError(Exception):
    """Exception from using a Result.

    The Result type is for type hints and reference only. It should not be constructed or returned.
    """

    def __init__(self) -> None:
        """Initialize a ResultError."""
        super().__init__("Do not use the Result type. Use Ok() or Err() instead.")


type Result[T, E] = Ok[T, E] | Err[T, E]


class Ok[T, E]:
    """Rust-like Ok type."""

    __match_args__ = ("inner",)

    def __init__(self, value: T) -> None:
        """Create an Ok object containing the value."""
        self.inner = value

    def __repr__(self) -> str:
        """Return a string representation of this Result."""
        return f"Ok({self.inner!r})"

    def is_ok(self) -> bool:
        """Return `True` if the result is `Ok`."""
        return True

    def is_ok_and(self, function: Callable[[T], bool]) -> bool:
        """Return `True` if the result is `Ok` and the value inside of it matches a predicate."""
        return function(self.inner)

    def is_err(self) -> bool:
        """Return `True` if the result is `Err`."""
        return False

    def is_err_and(self, _function: Callable[[E], bool]) -> bool:
        """Return `True` if the result is `Err` and the value inside of it matches a predicate."""
        return False

    def inspect(self, function: Callable[[T], None]) -> Ok[T, E]:
        """Call a funtion with a reference to the contained value of `Ok` and return self."""
        function(self.inner)
        return self

    def inspect_err(self, _function: Callable[[E], None]) -> Ok[T, E]:
        """Call a funtion with a reference to the contained value of `Err` and return self."""
        return self

    def ok(self) -> T | None:
        """Convert self into `T` or `None`, discarding a contained `Err`."""
        return self.inner

    def err(self) -> E | None:
        """Convert self into `E` or `None`, discarding a contained `Ok`."""
        return None

    def map[U](self, function: Callable[[T], U]) -> Ok[U, E]:
        """Map the contained `Ok` value from `T` to `U`, leaving an `Err` value untouched."""
        return Ok(function(self.inner))

    def map_or[U](self, _default: U, function: Callable[[T], U]) -> U:
        """Return the provided default if `Err`, or maps the contained value if `Ok`."""
        return function(self.inner)

    def map_err[U](self, _function: Callable[[E], U]) -> Ok[T, U]:
        """Map the contained `Err` value from `E` to `U`, leaving an `Ok` value untouched."""
        return Ok(self.inner)

    def expect(self, _message: str) -> T:
        """Return the contained `Ok` value, or raise a TypeError with the given message.

        For best practice, the message should be written with the word "should".
        """
        return self.inner

    def unwrap(self) -> T:
        """Return the contained Ok value, or raise a TypeError.

        Prefer `expect` over `unwrap` for better logging.
        """
        return self.inner

    def expect_err(self, message: str) -> E:
        """Return the contained `Err` value, or raise a TypeError with the given message.

        For best practice, the message should be written with the word "should".
        """
        msg = f"expect_err: {message}: {self!r}"
        raise TypeError(msg)

    def unwrap_err(self) -> E:
        """Return the contained Ok value, or raise a TypeError.

        Prefer `expect_err` over `unwrap_err` for better logging.
        """
        msg = f"unwrap_err: {self!r}"
        raise TypeError(msg)


class Err[T, E]:
    """Rust-like Err type."""

    __match_args__ = ("inner",)

    def __init__(self, value: E) -> None:
        """Create an Ok object containing the value."""
        self.inner = value

    def __repr__(self) -> str:
        """Return a string representation of this Result."""
        return f"Err({self.inner!r})"

    def is_ok(self) -> bool:
        """Return `True` if the result is `Ok`."""
        return False

    def is_ok_and(self, _function: Callable[[T], bool]) -> bool:
        """Return `True` if the result is `Ok` and the value inside of it matches a predicate."""
        return False

    def is_err(self) -> bool:
        """Return `True` if the result is `Err`."""
        return True

    def is_err_and(self, function: Callable[[E], bool]) -> bool:
        """Return `True` if the result is `Err` and the value inside of it matches a predicate."""
        return function(self.inner)

    def inspect(self, _function: Callable[[T], None]) -> Err[T, E]:
        """Call a funtion with a reference to the contained value of `Ok` and return self."""
        return self

    def inspect_err(self, function: Callable[[E], None]) -> Err[T, E]:
        """Call a funtion with a reference to the contained value of `Err` and return self."""
        function(self.inner)
        return self

    def ok(self) -> T | None:
        """Convert self into `T` or `None`, discarding a contained `Err`."""
        return None

    def err(self) -> E | None:
        """Convert self into `E` or `None`, discarding a contained `Ok`."""
        return self.inner

    def map[U](self, _function: Callable[[T], U]) -> Err[U, E]:
        """Map the contained `Ok` value from `T` to `U`, leaving an `Err` value untouched."""
        return Err(self.inner)

    def map_or[U](self, default: U, _function: Callable[[T], U]) -> U:
        """Return the provided default if `Err`, or maps the contained value if `Ok`."""
        return default

    def map_err[U](self, function: Callable[[E], U]) -> Err[T, U]:
        """Map the contained `Err` value from `E` to `U`, leaving an `Ok` value untouched."""
        return Err(function(self.inner))

    def expect(self, message: str) -> T:
        """Return the contained `Ok` value, or raise a TypeError with the given message.

        For best practice, the message should be written with the word "should".
        """
        msg = f"expect: {message}: {self!r}"
        raise TypeError(msg)

    def unwrap(self) -> T:
        """Return the contained Ok value, or raise a TypeError.

        Prefer `expect` over `unwrap` for better logging.
        """
        msg = f"unwrap: {self!r}"
        raise TypeError(msg)

    def expect_err(self, _message: str) -> E:
        """Return the contained `Err` value, or raise a TypeError with the given message.

        For best practice, the message should be written with the word "should".
        """
        return self.inner

    def unwrap_err(self) -> E:
        """Return the contained Ok value, or raise a TypeError.

        Prefer `expect_err` over `unwrap_err` for better logging.
        """
        return self.inner
