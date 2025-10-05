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


class Result[O, E]:
    """Rust-like Result type."""

    def __init__(self) -> None:
        """Create a Result object. THIS WILL RAISE. Use Ok() or Err() isntead."""
        raise ResultError

    def __repr__(self) -> str:
        """Return a string representation of this Result."""
        raise ResultError

    def is_ok(self) -> bool:
        """Return `True` if the result is `Ok`."""
        raise ResultError

    def is_ok_and(self, _function: Callable[[O], bool]) -> bool:
        """Return `True` if the result is `Ok` and the value inside of it matches a predicate."""
        raise ResultError

    def is_err(self) -> bool:
        """Return `True` if the result is `Err`."""
        raise ResultError

    def is_err_and(self, _function: Callable[[E], bool]) -> bool:
        """Return `True` if the result is `Err` and the value inside of it matches a predicate."""
        raise ResultError

    def inspect(self, _function: Callable[[O], None]) -> Result[O, E]:
        """Call a funtion with a reference to the contained value of `Ok` and return self."""
        raise ResultError

    def inspect_err(self, _function: Callable[[E], None]) -> Result[O, E]:
        """Call a funtion with a reference to the contained value of `Err` and return self."""
        raise ResultError

    def ok(self) -> O | None:
        """Convert self into `O` or `None`, discarding a contained `Err`."""
        raise ResultError

    def err(self) -> E | None:
        """Convert self into `E` or `None`, discarding a contained `Ok`."""
        raise ResultError

    def map[U](self, _function: Callable[[O], U]) -> Result[U, E]:
        """Map the contained `Ok` value from `O` to `U`, leaving an `Err` value untouched."""
        raise ResultError

    def map_or[U](self, _default: U, _function: Callable[[O], U]) -> Result[U, E]:
        """Return the provided default if `Err`, or maps the contained value if `Ok`."""
        raise ResultError

    def map_err[U](self, _function: Callable[[E], U]) -> Result[O, U]:
        """Map the contained `Err` value from `E` to `U`, leaving an `Ok` value untouched."""
        raise ResultError

    def expect(self, _message: str) -> O:
        """Return the contained `Ok` value, or raise a TypeError with the given message.

        For best practice, the message should be written with the word "should".
        """
        raise ResultError

    def unwrap(self) -> O:
        """Return the contained Ok value, or raise a TypeError.

        Prefer `expect` over `unwrap` for better logging.
        """
        raise ResultError

    def expect_err(self, _message: str) -> O:
        """Return the contained `Err` value, or raise a TypeError with the given message.

        For best practice, the message should be written with the word "should".
        """
        raise ResultError

    def unwrap_err(self) -> O:
        """Return the contained Ok value, or raise a TypeError.

        Prefer `expect_err` over `unwrap_err` for better logging.
        """
        raise ResultError


class Ok[O, E]:
    """Rust-like Ok type."""

    __match_args__ = ("inner",)

    def __init__(self, value: O) -> None:
        """Create an Ok object containing the value."""
        self.inner = value

    def __repr__(self) -> str:
        """Return a string representation of this Result."""
        return f"Ok({self.inner!r})"

    def is_ok(self) -> bool:
        """Return `True` if the result is `Ok`."""
        return True

    def is_ok_and(self, function: Callable[[O], bool]) -> bool:
        """Return `True` if the result is `Ok` and the value inside of it matches a predicate."""
        return function(self.inner)

    def is_err(self) -> bool:
        """Return `True` if the result is `Err`."""
        return False

    def is_err_and(self, _function: Callable[[E], bool]) -> bool:
        """Return `True` if the result is `Err` and the value inside of it matches a predicate."""
        return False

    def inspect(self, function: Callable[[O], None]) -> Result[O, E]:
        """Call a funtion with a reference to the contained value of `Ok` and return self."""
        function(self.inner)
        return self

    def inspect_err(self, _function: Callable[[E], None]) -> Result[O, E]:
        """Call a funtion with a reference to the contained value of `Err` and return self."""
        return self

    def ok(self) -> O | None:
        """Convert self into `O` or `None`, discarding a contained `Err`."""
        return self.inner

    def err(self) -> E | None:
        """Convert self into `E` or `None`, discarding a contained `Ok`."""
        return None

    def map[U](self, function: Callable[[O], U]) -> Result[U, E]:
        """Map the contained `Ok` value from `O` to `U`, leaving an `Err` value untouched."""
        return function(self.inner)

    def map_or[U](self, _default: U, function: Callable[[O], U]) -> Result[U, E]:
        """Return the provided default if `Err`, or maps the contained value if `Ok`."""
        return function(self.inner)

    def map_err[U](self, _function: Callable[[E], U]) -> Result[O, U]:
        """Map the contained `Err` value from `E` to `U`, leaving an `Ok` value untouched."""
        return self

    def expect(self, _message: str) -> O:
        """Return the contained `Ok` value, or raise a TypeError with the given message.

        For best practice, the message should be written with the word "should".
        """
        return self.inner

    def unwrap(self) -> O:
        """Return the contained Ok value, or raise a TypeError.

        Prefer `expect` over `unwrap` for better logging.
        """
        return self.inner

    def expect_err(self, message: str) -> O:
        """Return the contained `Err` value, or raise a TypeError with the given message.

        For best practice, the message should be written with the word "should".
        """
        msg = f"expect_err: {message}: {self!r}"
        raise TypeError(msg)

    def unwrap_err(self) -> O:
        """Return the contained Ok value, or raise a TypeError.

        Prefer `expect_err` over `unwrap_err` for better logging.
        """
        msg = f"unwrap_err: {self!r}"
        raise TypeError(msg)


class Err[O, E]:
    """Rust-like Err type."""

    __match_args__ = ("inner",)

    def __init__(self, value: O) -> None:
        """Create an Ok object containing the value."""
        self.inner = value

    def __repr__(self) -> str:
        """Return a string representation of this Result."""
        return f"Err({self.inner!r})"

    def is_ok(self) -> bool:
        """Return `True` if the result is `Ok`."""
        return False

    def is_ok_and(self, _function: Callable[[O], bool]) -> bool:
        """Return `True` if the result is `Ok` and the value inside of it matches a predicate."""
        return False

    def is_err(self) -> bool:
        """Return `True` if the result is `Err`."""
        return True

    def is_err_and(self, function: Callable[[E], bool]) -> bool:
        """Return `True` if the result is `Err` and the value inside of it matches a predicate."""
        return function(self.inner)

    def inspect(self, _function: Callable[[O], None]) -> Result[O, E]:
        """Call a funtion with a reference to the contained value of `Ok` and return self."""
        return self

    def inspect_err(self, function: Callable[[E], None]) -> Result[O, E]:
        """Call a funtion with a reference to the contained value of `Err` and return self."""
        function(self.inner)
        return self

    def ok(self) -> O | None:
        """Convert self into `O` or `None`, discarding a contained `Err`."""
        return None

    def err(self) -> E | None:
        """Convert self into `E` or `None`, discarding a contained `Ok`."""
        return self.inner

    def map[U](self, _function: Callable[[O], U]) -> Result[U, E]:
        """Map the contained `Ok` value from `O` to `U`, leaving an `Err` value untouched."""
        return self

    def map_or[U](self, default: U, _function: Callable[[O], U]) -> Result[U, E]:
        """Return the provided default if `Err`, or maps the contained value if `Ok`."""
        return default

    def map_err[U](self, function: Callable[[E], U]) -> Result[O, U]:
        """Map the contained `Err` value from `E` to `U`, leaving an `Ok` value untouched."""
        return function(self.inner)

    def expect(self, message: str) -> O:
        """Return the contained `Ok` value, or raise a TypeError with the given message.

        For best practice, the message should be written with the word "should".
        """
        msg = f"expect: {message}: {self!r}"
        raise TypeError(msg)

    def unwrap(self) -> O:
        """Return the contained Ok value, or raise a TypeError.

        Prefer `expect` over `unwrap` for better logging.
        """
        msg = f"unwrap: {self!r}"
        raise TypeError(msg)

    def expect_err(self, _message: str) -> O:
        """Return the contained `Err` value, or raise a TypeError with the given message.

        For best practice, the message should be written with the word "should".
        """
        return self.inner

    def unwrap_err(self) -> O:
        """Return the contained Ok value, or raise a TypeError.

        Prefer `expect_err` over `unwrap_err` for better logging.
        """
        return self.inner
