/**
 * The MIT License
 * Copyright (c) 2020 Guillem Castro
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */

use devenv_common::dependency::Dependency;
use devenv_common::error::Error;

/// Common trait for all providers of dependencies
pub trait DependencyProvider {

    /// Name of the provider
    fn name(&self) -> String;

    /// Search for the best matches for a given dependency
    fn search(&self, dependency: &Dependency) -> Result<Vec<Dependency>, Error>;

    /// Get information about an specific dependency. If it does not exist, an error could be returned.
    fn info(&self, dependency: &Dependency) -> Result<& dyn DependencyInfo, Error>;

    /// Install an specific dependency. If it does not exist, an error will be returned.
    fn install(&self, dependency: &Dependency) -> Result<(), Error>;

}

pub trait DependencyInfo {
    
    fn name(&self) -> String;

    fn description(&self) -> String;

    fn version(&self) -> String;

    fn provider(&self) -> &dyn DependencyProvider;

}